// use axum::{Router, middleware};
// use sea_orm::DatabaseConnection;
// //use crate::middlewares::{logger::logger_middleware, auth::auth_middleware};
// 
// pub fn create_app(db_conn: DatabaseConnection) -> Router {
//     Router::new()
//         // 全局中间件（所有接口生效）
//         //.layer(logger_middleware())
//         // 合并路由
//         .merge(crate::routes::user::routes())
//         // 对需要认证的接口添加鉴权中间件
//        // .route_layer(middleware::from_fn(auth_middleware))
//         // 注入状态
//         .with_state(db_conn)
// }
