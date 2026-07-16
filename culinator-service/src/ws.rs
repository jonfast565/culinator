use axum::{
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
};
use culinator_core::{Formula, PercentageView};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::sync::broadcast;

use crate::{
    ServiceState,
    auth::AccessPolicy,
    models::{
        CreateRecipeRequest, ExportRecipeRequest, FormulaCalculationRequest, MoveRecipeRequest,
        PercentageRequest, SaveRecipeBookRequest, SaveRecipeRequest, TranslateRecipeImagesRequest,
        UpdateImportSettingsRequest, ValidateRequest,
    },
    routes,
};

const PROTOCOL: &str = "culinator.v1";
const AUTH_PREFIX: &str = "culinator.auth.";

#[derive(Clone)]
pub struct WebSocketState {
    pub service: ServiceState,
    pub access: AccessPolicy,
    events: broadcast::Sender<ServerEvent>,
}

impl WebSocketState {
    pub fn new(service: ServiceState, access: AccessPolicy) -> Self {
        let (events, _) = broadcast::channel(128);
        Self {
            service,
            access,
            events,
        }
    }

    pub fn publish(&self, event: ServerEvent) {
        let _ = self.events.send(event);
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerEvent {
    pub event: String,
    pub payload: Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RpcRequest {
    id: String,
    method: String,
    #[serde(default)]
    params: Value,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RpcResponse {
    id: String,
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<RpcError>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RpcError {
    code: &'static str,
    message: String,
}

pub async fn upgrade(
    ws: WebSocketUpgrade,
    State(state): State<WebSocketState>,
    headers: HeaderMap,
) -> Response {
    if !state.access.is_origin_allowed(headers.get(header::ORIGIN)) {
        return (StatusCode::FORBIDDEN, "Origin is not allowed").into_response();
    }

    let Some(protocols) = headers
        .get(header::SEC_WEBSOCKET_PROTOCOL)
        .and_then(|v| v.to_str().ok())
    else {
        return (
            StatusCode::UNAUTHORIZED,
            "Missing WebSocket protocol authentication",
        )
            .into_response();
    };
    let offered = protocols.split(',').map(str::trim).collect::<Vec<_>>();
    let expected_auth = format!("{AUTH_PREFIX}{}", state.access.token());
    if !offered.contains(&PROTOCOL)
        || !offered
            .iter()
            .any(|value| constant_time_eq(value, &expected_auth))
    {
        return (StatusCode::UNAUTHORIZED, "Invalid launch token").into_response();
    }

    ws.protocols([PROTOCOL])
        .on_upgrade(move |socket| session(socket, state))
}

async fn session(mut socket: WebSocket, state: WebSocketState) {
    let mut events = state.events.subscribe();
    let ready = ServerEvent {
        event: "service.ready".to_owned(),
        payload: json!({ "apiVersion": "v1", "transport": "websocket" }),
    };
    if send_json(&mut socket, &ready).await.is_err() {
        return;
    }

    loop {
        tokio::select! {
            incoming = socket.recv() => {
                match incoming {
                    Some(Ok(Message::Text(text))) => {
                        let response = match serde_json::from_str::<RpcRequest>(&text) {
                            Ok(request) => dispatch(request, &state).await,
                            Err(error) => RpcResponse {
                                id: String::new(),
                                ok: false,
                                result: None,
                                error: Some(RpcError { code: "invalid_request", message: error.to_string() }),
                            },
                        };
                        if send_json(&mut socket, &response).await.is_err() { break; }
                    }
                    Some(Ok(Message::Ping(value))) => {
                        if socket.send(Message::Pong(value)).await.is_err() { break; }
                    }
                    Some(Ok(Message::Close(_))) | None | Some(Err(_)) => break,
                    _ => {}
                }
            }
            event = events.recv() => {
                match event {
                    Ok(event) => if send_json(&mut socket, &event).await.is_err() { break; },
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        }
    }
}

async fn dispatch(request: RpcRequest, state: &WebSocketState) -> RpcResponse {
    let result = dispatch_inner(&request.method, request.params, state).await;
    match result {
        Ok(value) => RpcResponse {
            id: request.id,
            ok: true,
            result: Some(value),
            error: None,
        },
        Err(message) => RpcResponse {
            id: request.id,
            ok: false,
            result: None,
            error: Some(RpcError {
                code: "operation_failed",
                message,
            }),
        },
    }
}

async fn dispatch_inner(
    method: &str,
    params: Value,
    state: &WebSocketState,
) -> Result<Value, String> {
    match method {
        "recipes.list" => {
            let axum::Json(value) = routes::recipes::list(State(state.service.clone()))
                .await
                .map_err(to_string)?;
            serde_json::to_value(value).map_err(to_string)
        }
        "recipes.get" => {
            let id = required_string(&params, "id")?;
            let axum::Json(value) =
                routes::recipes::get(axum::extract::Path(id), State(state.service.clone()))
                    .await
                    .map_err(to_string)?;
            serde_json::to_value(value).map_err(to_string)
        }
        "recipes.create" => {
            let book_id = params
                .get("bookId")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);
            let (_, axum::Json(value)) = routes::recipes::create(
                State(state.service.clone()),
                axum::Json(CreateRecipeRequest { book_id }),
            )
            .await
            .map_err(to_string)?;
            state.publish(ServerEvent {
                event: "recipes.changed".to_owned(),
                payload: json!({"kind":"created","id":value.id}),
            });
            serde_json::to_value(value).map_err(to_string)
        }
        "recipes.save" => {
            let id = required_string(&params, "id")?;
            let source_text = required_string(&params, "sourceText")?;
            let axum::Json(value) = routes::recipes::save(
                axum::extract::Path(id.clone()),
                State(state.service.clone()),
                axum::Json(SaveRecipeRequest { source_text }),
            )
            .await
            .map_err(to_string)?;
            state.publish(ServerEvent {
                event: "recipes.changed".to_owned(),
                payload: json!({"kind":"saved","id":id}),
            });
            serde_json::to_value(value).map_err(to_string)
        }
        "recipes.delete" => {
            let id = required_string(&params, "id")?;
            routes::recipes::delete(
                axum::extract::Path(id.clone()),
                State(state.service.clone()),
            )
            .await
            .map_err(to_string)?;
            state.publish(ServerEvent {
                event: "recipes.changed".to_owned(),
                payload: json!({"kind":"deleted","id":id}),
            });
            Ok(Value::Null)
        }
        "books.list" => {
            let axum::Json(value) = routes::books::list(State(state.service.clone()))
                .await
                .map_err(to_string)?;
            serde_json::to_value(value).map_err(to_string)
        }
        "books.create" => {
            let title = required_string(&params, "title")?;
            let symbol = params
                .get("symbol")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);
            let description = params
                .get("description")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);
            let (_, axum::Json(value)) = routes::books::create(
                State(state.service.clone()),
                axum::Json(SaveRecipeBookRequest {
                    title,
                    symbol,
                    description,
                }),
            )
            .await
            .map_err(to_string)?;
            state.publish(ServerEvent {
                event: "books.changed".to_owned(),
                payload: json!({"kind":"created","id":value.id}),
            });
            serde_json::to_value(value).map_err(to_string)
        }
        "books.update" => {
            let id = required_string(&params, "id")?;
            let title = required_string(&params, "title")?;
            let symbol = params
                .get("symbol")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);
            let description = params
                .get("description")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);
            let axum::Json(value) = routes::books::update(
                axum::extract::Path(id.clone()),
                State(state.service.clone()),
                axum::Json(SaveRecipeBookRequest {
                    title,
                    symbol,
                    description,
                }),
            )
            .await
            .map_err(to_string)?;
            state.publish(ServerEvent {
                event: "books.changed".to_owned(),
                payload: json!({"kind":"updated","id":id}),
            });
            serde_json::to_value(value).map_err(to_string)
        }
        "books.delete" => {
            let id = required_string(&params, "id")?;
            routes::books::delete(
                axum::extract::Path(id.clone()),
                State(state.service.clone()),
            )
            .await
            .map_err(to_string)?;
            state.publish(ServerEvent {
                event: "books.changed".to_owned(),
                payload: json!({"kind":"deleted","id":id}),
            });
            Ok(Value::Null)
        }
        "recipes.move" => {
            let id = required_string(&params, "id")?;
            let book_id = params
                .get("bookId")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);
            let position = params.get("position").and_then(Value::as_i64).unwrap_or(0);
            routes::books::move_recipe(
                axum::extract::Path(id.clone()),
                State(state.service.clone()),
                axum::Json(MoveRecipeRequest {
                    book_id: book_id.clone(),
                    position,
                }),
            )
            .await
            .map_err(to_string)?;
            state.publish(ServerEvent {
                event: "recipes.changed".to_owned(),
                payload: json!({"kind":"moved","id":id,"bookId":book_id}),
            });
            state.publish(ServerEvent {
                event: "books.changed".to_owned(),
                payload: json!({"kind":"membership"}),
            });
            Ok(Value::Null)
        }
        "recipes.export" => {
            let id = required_string(&params, "id")?;
            let options =
                serde_json::from_value(params.get("options").cloned().ok_or("Missing options")?)
                    .map_err(to_string)?;
            let axum::Json(value) = routes::exports::export_recipe(
                axum::extract::Path(id),
                State(state.service.clone()),
                axum::Json(ExportRecipeRequest { options }),
            )
            .await
            .map_err(to_string)?;
            serde_json::to_value(value).map_err(to_string)
        }
        "imports.settings.get" => {
            let axum::Json(value) = routes::imports::get_settings(State(state.service.clone()))
                .await
                .map_err(to_string)?;
            serde_json::to_value(value).map_err(to_string)
        }
        "imports.settings.update" => {
            let request: UpdateImportSettingsRequest =
                serde_json::from_value(params).map_err(to_string)?;
            let axum::Json(value) =
                routes::imports::update_settings(State(state.service.clone()), axum::Json(request))
                    .await
                    .map_err(to_string)?;
            serde_json::to_value(value).map_err(to_string)
        }
        "imports.translate" => {
            let request: TranslateRecipeImagesRequest =
                serde_json::from_value(params).map_err(to_string)?;
            let axum::Json(value) =
                routes::imports::translate(State(state.service.clone()), axum::Json(request))
                    .await
                    .map_err(to_string)?;
            serde_json::to_value(value).map_err(to_string)
        }
        "recipes.schedule" => {
            let source_text = required_string(&params, "sourceText")?;
            let options: culinator_models::ScheduleOptions = params
                .get("options")
                .cloned()
                .map(serde_json::from_value)
                .transpose()
                .map_err(to_string)?
                .unwrap_or_default();
            let value = state
                .service
                .schedules()
                .schedule_source(&source_text, &options)
                .map_err(to_string)?;
            serde_json::to_value(value).map_err(to_string)
        }
        "recipes.validate" => {
            let source_text = required_string(&params, "sourceText")?;
            let axum::Json(value) = routes::recipes::validate(
                State(state.service.clone()),
                axum::Json(ValidateRequest { source_text }),
            )
            .await;
            serde_json::to_value(value).map_err(to_string)
        }
        "formulas.calculate" => {
            let formula: Formula =
                serde_json::from_value(params.get("formula").cloned().ok_or("Missing formula")?)
                    .map_err(to_string)?;
            let target_mass_grams = required_f64(&params, "targetMassGrams")?;
            let axum::Json(value) = routes::formulas::calculate(
                State(state.service.clone()),
                axum::Json(FormulaCalculationRequest {
                    formula,
                    target_mass_grams,
                }),
            )
            .await
            .map_err(to_string)?;
            serde_json::to_value(value).map_err(to_string)
        }
        "formulas.percentages" => {
            let formula: Formula =
                serde_json::from_value(params.get("formula").cloned().ok_or("Missing formula")?)
                    .map_err(to_string)?;
            let view: PercentageView =
                serde_json::from_value(params.get("view").cloned().ok_or("Missing view")?)
                    .map_err(to_string)?;
            let axum::Json(value) = routes::formulas::percentages(
                State(state.service.clone()),
                axum::Json(PercentageRequest { formula, view }),
            )
            .await
            .map_err(to_string)?;
            serde_json::to_value(value).map_err(to_string)
        }
        "formulas.save" => {
            let formula: Formula =
                serde_json::from_value(params.get("formula").cloned().unwrap_or(params))
                    .map_err(to_string)?;
            let axum::Json(value) =
                routes::formulas::save(State(state.service.clone()), axum::Json(formula))
                    .await
                    .map_err(to_string)?;
            state.publish(ServerEvent {
                event: "formulas.changed".to_owned(),
                payload: json!({"recipeId":value.recipe_id,"formulaId":value.id}),
            });
            serde_json::to_value(value).map_err(to_string)
        }
        "formulas.list" => {
            let recipe_id = required_string(&params, "recipeId")?;
            let axum::Json(value) = routes::formulas::list_for_recipe(
                axum::extract::Path(recipe_id),
                State(state.service.clone()),
            )
            .await
            .map_err(to_string)?;
            serde_json::to_value(value).map_err(to_string)
        }
        "formulas.get" => {
            let formula_id = required_string(&params, "formulaId")?;
            let axum::Json(value) = routes::formulas::get(
                axum::extract::Path(formula_id),
                State(state.service.clone()),
            )
            .await
            .map_err(to_string)?;
            serde_json::to_value(value).map_err(to_string)
        }
        "haccp.list" => {
            let recipe_id = required_string(&params, "recipeId")?;
            let axum::Json(value) = routes::haccp::list_for_recipe(
                axum::extract::Path(recipe_id),
                State(state.service.clone()),
            )
            .await
            .map_err(to_string)?;
            serde_json::to_value(value).map_err(to_string)
        }
        "haccp.get" => {
            let plan_id = required_string(&params, "planId")?;
            let axum::Json(value) =
                routes::haccp::get(axum::extract::Path(plan_id), State(state.service.clone()))
                    .await
                    .map_err(to_string)?;
            serde_json::to_value(value).map_err(to_string)
        }
        "haccp.create" => {
            let recipe_id = required_string(&params, "recipeId")?;
            let title = required_string(&params, "title")?;
            let description = params
                .get("description")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);
            let (_, axum::Json(value)) = routes::haccp::create(
                axum::extract::Path(recipe_id.clone()),
                State(state.service.clone()),
                axum::Json(culinator_models::NewHaccpPlan { title, description }),
            )
            .await
            .map_err(to_string)?;
            state.publish(ServerEvent {
                event: "haccp.changed".to_owned(),
                payload: json!({"kind":"created","recipeId":recipe_id,"planId":value.id}),
            });
            serde_json::to_value(value).map_err(to_string)
        }
        "haccp.save" => {
            let plan_id = required_string(&params, "planId")?;
            let request: culinator_models::SaveHaccpPlanRequest =
                serde_json::from_value(params).map_err(to_string)?;
            let axum::Json(value) = routes::haccp::save(
                axum::extract::Path(plan_id.clone()),
                State(state.service.clone()),
                axum::Json(request),
            )
            .await
            .map_err(to_string)?;
            state.publish(ServerEvent {
                event: "haccp.changed".to_owned(),
                payload: json!({"kind":"saved","planId":plan_id,"recipeId":value.recipe_id}),
            });
            serde_json::to_value(value).map_err(to_string)
        }
        "haccp.delete" => {
            let plan_id = required_string(&params, "planId")?;
            routes::haccp::delete(
                axum::extract::Path(plan_id.clone()),
                State(state.service.clone()),
            )
            .await
            .map_err(to_string)?;
            state.publish(ServerEvent {
                event: "haccp.changed".to_owned(),
                payload: json!({"kind":"deleted","planId":plan_id}),
            });
            Ok(Value::Null)
        }
        "haccp.record" => {
            let ccp_id = required_string(&params, "ccpId")?;
            let request: culinator_models::NewHaccpMonitoringRecord =
                serde_json::from_value(params).map_err(to_string)?;
            let (_, axum::Json(value)) = routes::haccp::add_monitoring_record(
                axum::extract::Path(ccp_id.clone()),
                State(state.service.clone()),
                axum::Json(request),
            )
            .await
            .map_err(to_string)?;
            state.publish(ServerEvent {
                event: "haccp.changed".to_owned(),
                payload: json!({"kind":"recorded","ccpId":ccp_id,"recordId":value.id}),
            });
            serde_json::to_value(value).map_err(to_string)
        }
        "tries.list" => {
            let recipe_id = required_string(&params, "recipeId")?;
            let axum::Json(value) = routes::kitchen::list_for_recipe(
                axum::extract::Path(recipe_id),
                State(state.service.clone()),
            )
            .await
            .map_err(to_string)?;
            serde_json::to_value(value).map_err(to_string)
        }
        "tries.get" => {
            let try_id = required_string(&params, "tryId")?;
            let axum::Json(value) =
                routes::kitchen::get(axum::extract::Path(try_id), State(state.service.clone()))
                    .await
                    .map_err(to_string)?;
            serde_json::to_value(value).map_err(to_string)
        }
        "tries.start" => {
            let recipe_id = required_string(&params, "recipeId")?;
            let request: culinator_models::NewRecipeTry =
                serde_json::from_value(params).map_err(to_string)?;
            let (_, axum::Json(value)) = routes::kitchen::start(
                axum::extract::Path(recipe_id.clone()),
                State(state.service.clone()),
                axum::Json(request),
            )
            .await
            .map_err(to_string)?;
            state.publish(ServerEvent {
                event: "tries.changed".to_owned(),
                payload: json!({"kind":"started","recipeId":recipe_id,"tryId":value.id}),
            });
            serde_json::to_value(value).map_err(to_string)
        }
        "tries.update" => {
            let try_id = required_string(&params, "tryId")?;
            let request: culinator_models::UpdateRecipeTry =
                serde_json::from_value(params).map_err(to_string)?;
            let axum::Json(value) = routes::kitchen::update(
                axum::extract::Path(try_id.clone()),
                State(state.service.clone()),
                axum::Json(request),
            )
            .await
            .map_err(to_string)?;
            state.publish(ServerEvent {
                event: "tries.changed".to_owned(),
                payload: json!({"kind":"updated","tryId":try_id}),
            });
            serde_json::to_value(value).map_err(to_string)
        }
        "tries.updateOperation" => {
            let try_id = required_string(&params, "tryId")?;
            let operation_id = required_string(&params, "operationId")?;
            let request: culinator_models::UpdateTryOperation =
                serde_json::from_value(params).map_err(to_string)?;
            let axum::Json(value) = routes::kitchen::update_operation(
                axum::extract::Path((try_id.clone(), operation_id.clone())),
                State(state.service.clone()),
                axum::Json(request),
            )
            .await
            .map_err(to_string)?;
            state.publish(ServerEvent {
                event: "tries.changed".to_owned(),
                payload: json!({"kind":"operation","tryId":try_id,"operationId":operation_id}),
            });
            serde_json::to_value(value).map_err(to_string)
        }
        "tries.observe" => {
            let try_id = required_string(&params, "tryId")?;
            let request: culinator_models::NewTryObservation =
                serde_json::from_value(params).map_err(to_string)?;
            let axum::Json(value) = routes::kitchen::add_observation(
                axum::extract::Path(try_id.clone()),
                State(state.service.clone()),
                axum::Json(request),
            )
            .await
            .map_err(to_string)?;
            state.publish(ServerEvent {
                event: "tries.changed".to_owned(),
                payload: json!({"kind":"observed","tryId":try_id}),
            });
            serde_json::to_value(value).map_err(to_string)
        }
        "tries.delete" => {
            let try_id = required_string(&params, "tryId")?;
            routes::kitchen::delete(
                axum::extract::Path(try_id.clone()),
                State(state.service.clone()),
            )
            .await
            .map_err(to_string)?;
            state.publish(ServerEvent {
                event: "tries.changed".to_owned(),
                payload: json!({"kind":"deleted","tryId":try_id}),
            });
            Ok(Value::Null)
        }
        "nutrition.status" => {
            let axum::Json(value) = routes::nutrition::status(State(state.service.clone()))
                .await
                .map_err(to_string)?;
            serde_json::to_value(value).map_err(to_string)
        }
        "nutrition.search" => {
            let query = required_string(&params, "query")?;
            let limit = params.get("limit").and_then(Value::as_u64).unwrap_or(20) as usize;
            let axum::Json(value) = routes::nutrition::search(
                axum::extract::Query(routes::nutrition::SearchQuery { q: query, limit }),
                State(state.service.clone()),
            )
            .await
            .map_err(to_string)?;
            serde_json::to_value(value).map_err(to_string)
        }
        "nutrition.listLinks" => {
            let recipe_id = required_string(&params, "recipeId")?;
            let axum::Json(value) = routes::nutrition::list_links(
                axum::extract::Path(recipe_id),
                State(state.service.clone()),
            )
            .await
            .map_err(to_string)?;
            serde_json::to_value(value).map_err(to_string)
        }
        "nutrition.link" => {
            let recipe_id = required_string(&params, "recipeId")?;
            let request: culinator_models::LinkResourceNutritionRequest =
                serde_json::from_value(params).map_err(to_string)?;
            let (_, axum::Json(value)) = routes::nutrition::link_resource(
                axum::extract::Path(recipe_id.clone()),
                State(state.service.clone()),
                axum::Json(request),
            )
            .await
            .map_err(to_string)?;
            state.publish(ServerEvent {
                event: "nutrition.changed".to_owned(),
                payload: json!({"kind":"linked","recipeId":recipe_id,"resourceSymbol":value.resource_symbol}),
            });
            serde_json::to_value(value).map_err(to_string)
        }
        "nutrition.unlink" => {
            let recipe_id = required_string(&params, "recipeId")?;
            let resource_symbol = required_string(&params, "resourceSymbol")?;
            routes::nutrition::unlink_resource(
                axum::extract::Path((recipe_id.clone(), resource_symbol.clone())),
                State(state.service.clone()),
            )
            .await
            .map_err(to_string)?;
            state.publish(ServerEvent {
                event: "nutrition.changed".to_owned(),
                payload: json!({"kind":"unlinked","recipeId":recipe_id,"resourceSymbol":resource_symbol}),
            });
            Ok(Value::Null)
        }
        "nutrition.calculate" => {
            let recipe_id = required_string(&params, "recipeId")?;
            let request: culinator_models::CalculateRecipeNutritionRequest =
                serde_json::from_value(params).map_err(to_string)?;
            let axum::Json(value) = routes::nutrition::calculate(
                axum::extract::Path(recipe_id),
                State(state.service.clone()),
                axum::Json(request),
            )
            .await
            .map_err(to_string)?;
            serde_json::to_value(value).map_err(to_string)
        }
        "service.ping" => Ok(json!({"status":"ok"})),
        _ => Err(format!("Unknown RPC method: {method}")),
    }
}

async fn send_json<T: Serialize>(socket: &mut WebSocket, value: &T) -> Result<(), axum::Error> {
    let text = serde_json::to_string(value).map_err(axum::Error::new)?;
    socket.send(Message::Text(text.into())).await
}

fn required_string(value: &Value, key: &str) -> Result<String, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| format!("Missing {key}"))
}

fn required_f64(value: &Value, key: &str) -> Result<f64, String> {
    value
        .get(key)
        .and_then(Value::as_f64)
        .ok_or_else(|| format!("Missing {key}"))
}

fn to_string(error: impl std::fmt::Display) -> String {
    error.to_string()
}

fn constant_time_eq(left: &str, right: &str) -> bool {
    let left = left.as_bytes();
    let right = right.as_bytes();
    if left.len() != right.len() {
        return false;
    }
    left.iter()
        .zip(right)
        .fold(0_u8, |difference, (a, b)| difference | (a ^ b))
        == 0
}
#[cfg(test)]
mod test;
