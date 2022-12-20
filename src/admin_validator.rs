use actix_web::dev::ServiceRequest;
use actix_web_httpauth::extractors::{
    bearer::{BearerAuth, Config},
    AuthenticationError,
};

pub(crate) async fn admin_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (actix_web::Error, ServiceRequest)> {
    let token = std::env::var("ADMIN_BEARER_TOKEN").unwrap();

    if credentials.token() == token {
        Ok(req)
    } else {
        let config = req.app_data::<Config>().cloned().unwrap_or_default();

        Err((AuthenticationError::from(config).into(), req))
    }
}
