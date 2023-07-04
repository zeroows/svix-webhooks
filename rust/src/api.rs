use crate::apis::configuration::Configuration;
use crate::apis::{
    application_api, authentication_api, background_tasks_api, endpoint_api, event_type_api,
    integration_api, message_api, message_attempt_api,
};
use crate::error::Result;
pub use crate::models::*;

const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
pub struct SvixOptions {
    pub debug: bool,
    pub server_url: Option<String>,
}

pub struct Svix {
    cfg: Configuration,
}

impl Svix {
    pub fn new(token: String, options: Option<SvixOptions>) -> Self {
        let base_path = options.and_then(|x| x.server_url).unwrap_or_else(|| {
            match token.split('.').last() {
                Some("us") => "https://api.us.svix.com",
                Some("eu") => "https://api.eu.svix.com",
                Some("in") => "https://api.in.svix.com",
                _ => "https://api.svix.com",
            }
            .to_string()
        });
        let cfg = Configuration {
            base_path,
            user_agent: Some(format!("svix-libs/{CRATE_VERSION}/rust")),
            bearer_access_token: Some(token),
            ..Configuration::default()
        };

        Self { cfg }
    }

    pub fn authentication(&self) -> Authentication<'_> {
        Authentication::new(&self.cfg)
    }

    pub fn application(&self) -> Application<'_> {
        Application::new(&self.cfg)
    }

    pub fn endpoint(&self) -> Endpoint<'_> {
        Endpoint::new(&self.cfg)
    }

    pub fn integration(&self) -> Integration<'_> {
        Integration::new(&self.cfg)
    }

    pub fn event_type(&self) -> EventType<'_> {
        EventType::new(&self.cfg)
    }

    pub fn message(&self) -> Message<'_> {
        Message::new(&self.cfg)
    }

    pub fn message_attempt(&self) -> MessageAttempt<'_> {
        MessageAttempt::new(&self.cfg)
    }
}

#[derive(Default)]
pub struct PostOptions {
    pub idempotency_key: Option<String>,
}

pub struct Authentication<'a> {
    cfg: &'a Configuration,
}

impl<'a> Authentication<'a> {
    fn new(cfg: &'a Configuration) -> Self {
        Self { cfg }
    }

    pub async fn dashboard_access(
        &self,
        app_id: String,
        options: Option<PostOptions>,
    ) -> Result<DashboardAccessOut> {
        let options = options.unwrap_or_default();
        Ok(authentication_api::v1_authentication_dashboard_access(
            self.cfg,
            authentication_api::V1AuthenticationDashboardAccessParams {
                app_id,
                idempotency_key: options.idempotency_key,
            },
        )
        .await?)
    }

    pub async fn app_portal_access(
        &self,
        app_id: String,
        app_portal_access_in: AppPortalAccessIn,
        options: Option<PostOptions>,
    ) -> Result<AppPortalAccessOut> {
        let options = options.unwrap_or_default();
        Ok(authentication_api::v1_authentication_app_portal_access(
            self.cfg,
            authentication_api::V1AuthenticationAppPortalAccessParams {
                app_id,
                app_portal_access_in,
                idempotency_key: options.idempotency_key,
            },
        )
        .await?)
    }

    pub async fn logout(&self, options: Option<PostOptions>) -> Result<()> {
        let PostOptions { idempotency_key } = options.unwrap_or_default();
        Ok(authentication_api::v1_authentication_logout(
            self.cfg,
            authentication_api::V1AuthenticationLogoutParams { idempotency_key },
        )
        .await?)
    }
}

#[derive(Default)]
pub struct ListOptions {
    pub iterator: Option<String>,
    pub limit: Option<i32>,
}

#[derive(Default)]
pub struct ApplicationListOptions {
    pub iterator: Option<String>,
    pub limit: Option<i32>,
    pub order: Option<Ordering>,
}

pub struct Application<'a> {
    cfg: &'a Configuration,
}

impl<'a> Application<'a> {
    fn new(cfg: &'a Configuration) -> Self {
        Self { cfg }
    }

    pub async fn list(
        &self,
        options: Option<ApplicationListOptions>,
    ) -> Result<ListResponseApplicationOut> {
        let ApplicationListOptions {
            iterator,
            limit,
            order,
        } = options.unwrap_or_default();
        Ok(application_api::v1_application_list(
            self.cfg,
            application_api::V1ApplicationListParams {
                iterator,
                limit,
                order,
            },
        )
        .await?)
    }

    pub async fn create(
        &self,
        application_in: ApplicationIn,
        options: Option<PostOptions>,
    ) -> Result<ApplicationOut> {
        let PostOptions { idempotency_key } = options.unwrap_or_default();
        Ok(application_api::v1_application_create(
            self.cfg,
            application_api::V1ApplicationCreateParams {
                application_in,
                idempotency_key,
                get_if_exists: None,
            },
        )
        .await?)
    }

    pub async fn get_or_create(
        &self,
        application_in: ApplicationIn,
        options: Option<PostOptions>,
    ) -> Result<ApplicationOut> {
        let PostOptions { idempotency_key } = options.unwrap_or_default();
        Ok(application_api::v1_application_create(
            self.cfg,
            application_api::V1ApplicationCreateParams {
                application_in,
                idempotency_key,
                get_if_exists: Some(true),
            },
        )
        .await?)
    }

    pub async fn get(&self, app_id: String) -> Result<ApplicationOut> {
        Ok(application_api::v1_application_get(
            self.cfg,
            application_api::V1ApplicationGetParams { app_id },
        )
        .await?)
    }

    pub async fn update(
        &self,
        app_id: String,
        application_in: ApplicationIn,
        _: Option<PostOptions>,
    ) -> Result<ApplicationOut> {
        Ok(application_api::v1_application_update(
            self.cfg,
            application_api::V1ApplicationUpdateParams {
                app_id,
                application_in,
            },
        )
        .await?)
    }

    pub async fn delete(&self, app_id: String) -> Result<()> {
        Ok(application_api::v1_application_delete(
            self.cfg,
            application_api::V1ApplicationDeleteParams { app_id },
        )
        .await?)
    }
}

#[derive(Default)]
pub struct EndpointListOptions {
    pub iterator: Option<String>,
    pub limit: Option<i32>,
    pub order: Option<Ordering>,
}

pub struct Endpoint<'a> {
    cfg: &'a Configuration,
}

#[derive(Default)]
pub struct EndpointStatsOptions {
    pub since: Option<String>,
    pub until: Option<String>,
}

impl<'a> Endpoint<'a> {
    fn new(cfg: &'a Configuration) -> Self {
        Self { cfg }
    }

    pub async fn list(
        &self,
        app_id: String,
        options: Option<EndpointListOptions>,
    ) -> Result<ListResponseEndpointOut> {
        let EndpointListOptions {
            iterator,
            limit,
            order,
        } = options.unwrap_or_default();
        Ok(endpoint_api::v1_endpoint_list(
            self.cfg,
            endpoint_api::V1EndpointListParams {
                app_id,
                order,
                iterator,
                limit,
            },
        )
        .await?)
    }

    pub async fn create(
        &self,
        app_id: String,
        endpoint_in: EndpointIn,
        options: Option<PostOptions>,
    ) -> Result<EndpointOut> {
        let PostOptions { idempotency_key } = options.unwrap_or_default();
        Ok(endpoint_api::v1_endpoint_create(
            self.cfg,
            endpoint_api::V1EndpointCreateParams {
                app_id,
                endpoint_in,
                idempotency_key,
            },
        )
        .await?)
    }

    pub async fn get(&self, app_id: String, endpoint_id: String) -> Result<EndpointOut> {
        Ok(endpoint_api::v1_endpoint_get(
            self.cfg,
            endpoint_api::V1EndpointGetParams {
                app_id,
                endpoint_id,
            },
        )
        .await?)
    }

    pub async fn update(
        &self,
        app_id: String,
        endpoint_id: String,
        endpoint_update: EndpointUpdate,
        _: Option<PostOptions>,
    ) -> Result<EndpointOut> {
        Ok(endpoint_api::v1_endpoint_update(
            self.cfg,
            endpoint_api::V1EndpointUpdateParams {
                app_id,
                endpoint_id,
                endpoint_update,
            },
        )
        .await?)
    }

    pub async fn delete(&self, app_id: String, endpoint_id: String) -> Result<()> {
        Ok(endpoint_api::v1_endpoint_delete(
            self.cfg,
            endpoint_api::V1EndpointDeleteParams {
                app_id,
                endpoint_id,
            },
        )
        .await?)
    }

    pub async fn get_secret(
        &self,
        app_id: String,
        endpoint_id: String,
    ) -> Result<EndpointSecretOut> {
        Ok(endpoint_api::v1_endpoint_get_secret(
            self.cfg,
            endpoint_api::V1EndpointGetSecretParams {
                app_id,
                endpoint_id,
            },
        )
        .await?)
    }

    pub async fn rotate_secret(
        &self,
        app_id: String,
        endpoint_id: String,
        endpoint_secret_rotate_in: EndpointSecretRotateIn,
    ) -> Result<()> {
        Ok(endpoint_api::v1_endpoint_rotate_secret(
            self.cfg,
            endpoint_api::V1EndpointRotateSecretParams {
                app_id,
                endpoint_id,
                endpoint_secret_rotate_in,
                idempotency_key: None,
            },
        )
        .await?)
    }

    pub async fn recover(
        &self,
        app_id: String,
        endpoint_id: String,
        recover_in: RecoverIn,
    ) -> Result<()> {
        endpoint_api::v1_endpoint_recover(
            self.cfg,
            endpoint_api::V1EndpointRecoverParams {
                app_id,
                endpoint_id,
                recover_in,
                idempotency_key: None,
            },
        )
        .await?;
        Ok(())
    }

    pub async fn get_headers(
        &self,
        app_id: String,
        endpoint_id: String,
    ) -> Result<EndpointHeadersOut> {
        Ok(endpoint_api::v1_endpoint_get_headers(
            self.cfg,
            endpoint_api::V1EndpointGetHeadersParams {
                app_id,
                endpoint_id,
            },
        )
        .await?)
    }

    pub async fn update_headers(
        &self,
        app_id: String,
        endpoint_id: String,
        endpoint_headers_in: EndpointHeadersIn,
    ) -> Result<()> {
        Ok(endpoint_api::v1_endpoint_update_headers(
            self.cfg,
            endpoint_api::V1EndpointUpdateHeadersParams {
                app_id,
                endpoint_id,
                endpoint_headers_in,
            },
        )
        .await?)
    }

    pub async fn patch_headers(
        &self,
        app_id: String,
        endpoint_id: String,
        endpoint_headers_patch_in: EndpointHeadersPatchIn,
    ) -> Result<()> {
        Ok(endpoint_api::v1_endpoint_patch_headers(
            self.cfg,
            endpoint_api::V1EndpointPatchHeadersParams {
                app_id,
                endpoint_id,
                endpoint_headers_patch_in,
            },
        )
        .await?)
    }

    pub async fn get_stats(
        &self,
        app_id: String,
        endpoint_id: String,
        options: Option<EndpointStatsOptions>,
    ) -> Result<EndpointStats> {
        let EndpointStatsOptions { since, until } = options.unwrap_or_default();
        Ok(endpoint_api::v1_endpoint_get_stats(
            self.cfg,
            endpoint_api::V1EndpointGetStatsParams {
                app_id,
                endpoint_id,
                since,
                until,
            },
        )
        .await?)
    }

    pub async fn replay_missing(
        &self,
        app_id: String,
        endpoint_id: String,
        replay_in: ReplayIn,
        options: Option<PostOptions>,
    ) -> Result<()> {
        let PostOptions { idempotency_key } = options.unwrap_or_default();
        endpoint_api::v1_endpoint_replay(
            self.cfg,
            endpoint_api::V1EndpointReplayParams {
                app_id,
                endpoint_id,
                replay_in,
                idempotency_key,
            },
        )
        .await?;
        Ok(())
    }

    pub async fn transformation_get(
        &self,
        app_id: String,
        endpoint_id: String,
    ) -> Result<EndpointTransformationOut> {
        Ok(endpoint_api::v1_endpoint_transformation_get(
            self.cfg,
            endpoint_api::V1EndpointTransformationGetParams {
                app_id,
                endpoint_id,
            },
        )
        .await?)
    }

    pub async fn transformation_partial_update(
        &self,
        app_id: String,
        endpoint_id: String,
        endpoint_transformation_in: EndpointTransformationIn,
    ) -> Result<()> {
        endpoint_api::v1_endpoint_transformation_partial_update(
            self.cfg,
            endpoint_api::V1EndpointTransformationPartialUpdateParams {
                app_id,
                endpoint_id,
                endpoint_transformation_in,
            },
        )
        .await?;
        Ok(())
    }

    pub async fn send_example(
        &self,
        app_id: String,
        endpoint_id: String,
        event_example_in: EventExampleIn,
        options: Option<PostOptions>,
    ) -> Result<MessageOut> {
        let PostOptions { idempotency_key } = options.unwrap_or_default();
        Ok(endpoint_api::v1_endpoint_send_example(
            self.cfg,
            endpoint_api::V1EndpointSendExampleParams {
                app_id,
                endpoint_id,
                event_example_in,
                idempotency_key,
            },
        )
        .await?)
    }
}

pub type IntegrationListOptions = ListOptions;

pub struct Integration<'a> {
    cfg: &'a Configuration,
}

impl<'a> Integration<'a> {
    fn new(cfg: &'a Configuration) -> Self {
        Self { cfg }
    }

    pub async fn list(
        &self,
        app_id: String,
        options: Option<IntegrationListOptions>,
    ) -> Result<ListResponseIntegrationOut> {
        let IntegrationListOptions { iterator, limit } = options.unwrap_or_default();
        Ok(integration_api::v1_integration_list(
            self.cfg,
            integration_api::V1IntegrationListParams {
                app_id,
                iterator,
                limit,
            },
        )
        .await?)
    }

    pub async fn create(
        &self,
        app_id: String,
        integration_in: IntegrationIn,
        options: Option<PostOptions>,
    ) -> Result<IntegrationOut> {
        let PostOptions { idempotency_key } = options.unwrap_or_default();
        Ok(integration_api::v1_integration_create(
            self.cfg,
            integration_api::V1IntegrationCreateParams {
                app_id,
                integration_in,
                idempotency_key,
            },
        )
        .await?)
    }

    pub async fn get(&self, app_id: String, integ_id: String) -> Result<IntegrationOut> {
        Ok(integration_api::v1_integration_get(
            self.cfg,
            integration_api::V1IntegrationGetParams { app_id, integ_id },
        )
        .await?)
    }

    pub async fn update(
        &self,
        app_id: String,
        integ_id: String,
        integration_update: IntegrationUpdate,
        _: Option<PostOptions>,
    ) -> Result<IntegrationOut> {
        Ok(integration_api::v1_integration_update(
            self.cfg,
            integration_api::V1IntegrationUpdateParams {
                app_id,
                integ_id,
                integration_update,
            },
        )
        .await?)
    }

    pub async fn delete(&self, app_id: String, integ_id: String) -> Result<()> {
        Ok(integration_api::v1_integration_delete(
            self.cfg,
            integration_api::V1IntegrationDeleteParams { app_id, integ_id },
        )
        .await?)
    }

    pub async fn get_key(&self, app_id: String, integ_id: String) -> Result<IntegrationKeyOut> {
        Ok(integration_api::v1_integration_get_key(
            self.cfg,
            integration_api::V1IntegrationGetKeyParams { app_id, integ_id },
        )
        .await?)
    }

    pub async fn rotate_key(&self, app_id: String, integ_id: String) -> Result<IntegrationKeyOut> {
        Ok(integration_api::v1_integration_rotate_key(
            self.cfg,
            integration_api::V1IntegrationRotateKeyParams {
                app_id,
                integ_id,
                idempotency_key: None,
            },
        )
        .await?)
    }
}

#[derive(Default)]
pub struct EventTypeListOptions {
    pub iterator: Option<String>,
    pub limit: Option<i32>,
    pub with_content: Option<bool>,
    pub include_archived: Option<bool>,
}

pub struct EventType<'a> {
    cfg: &'a Configuration,
}

impl<'a> EventType<'a> {
    fn new(cfg: &'a Configuration) -> Self {
        Self { cfg }
    }

    pub async fn list(
        &self,
        options: Option<EventTypeListOptions>,
    ) -> Result<ListResponseEventTypeOut> {
        let EventTypeListOptions {
            iterator,
            limit,
            with_content,
            include_archived,
        } = options.unwrap_or_default();
        Ok(event_type_api::v1_event_type_list(
            self.cfg,
            event_type_api::V1EventTypeListParams {
                iterator,
                limit,
                with_content,
                include_archived,
                order: None,
            },
        )
        .await?)
    }

    pub async fn create(
        &self,
        event_type_in: EventTypeIn,
        options: Option<PostOptions>,
    ) -> Result<EventTypeOut> {
        let PostOptions { idempotency_key } = options.unwrap_or_default();
        Ok(event_type_api::v1_event_type_create(
            self.cfg,
            event_type_api::V1EventTypeCreateParams {
                event_type_in,
                idempotency_key,
            },
        )
        .await?)
    }

    pub async fn get(&self, event_type_name: String) -> Result<EventTypeOut> {
        Ok(event_type_api::v1_event_type_get(
            self.cfg,
            event_type_api::V1EventTypeGetParams { event_type_name },
        )
        .await?)
    }

    pub async fn update(
        &self,
        event_type_name: String,
        event_type_update: EventTypeUpdate,
        _: Option<PostOptions>,
    ) -> Result<EventTypeOut> {
        Ok(event_type_api::v1_event_type_update(
            self.cfg,
            event_type_api::V1EventTypeUpdateParams {
                event_type_name,
                event_type_update,
            },
        )
        .await?)
    }

    pub async fn delete(&self, event_type_name: String) -> Result<()> {
        Ok(event_type_api::v1_event_type_delete(
            self.cfg,
            event_type_api::V1EventTypeDeleteParams {
                event_type_name,
                expunge: None,
            },
        )
        .await?)
    }
}

#[derive(Default)]
pub struct MessageListOptions {
    pub iterator: Option<String>,
    pub limit: Option<i32>,
    pub event_types: Option<Vec<String>>,
    // FIXME: make before and after actual dates
    /// RFC3339 date string
    pub before: Option<String>,
    /// RFC3339 date string
    pub after: Option<String>,
    pub channel: Option<String>,
    pub with_content: Option<bool>,
}

pub struct Message<'a> {
    cfg: &'a Configuration,
}

impl<'a> Message<'a> {
    fn new(cfg: &'a Configuration) -> Self {
        Self { cfg }
    }

    pub async fn list(
        &self,
        app_id: String,
        options: Option<MessageListOptions>,
    ) -> Result<ListResponseMessageOut> {
        let MessageListOptions {
            iterator,
            limit,
            event_types,
            before,
            after,
            channel,
            with_content,
        } = options.unwrap_or_default();
        Ok(message_api::v1_message_list(
            self.cfg,
            message_api::V1MessageListParams {
                app_id,
                iterator,
                limit,
                event_types,
                before,
                after,
                channel,
                with_content,
            },
        )
        .await?)
    }

    pub async fn create(
        &self,
        app_id: String,
        message_in: MessageIn,
        options: Option<PostOptions>,
    ) -> Result<MessageOut> {
        let PostOptions { idempotency_key } = options.unwrap_or_default();
        Ok(message_api::v1_message_create(
            self.cfg,
            message_api::V1MessageCreateParams {
                app_id,
                message_in,
                idempotency_key,
                with_content: None,
            },
        )
        .await?)
    }

    pub async fn get(&self, app_id: String, msg_id: String) -> Result<MessageOut> {
        Ok(message_api::v1_message_get(
            self.cfg,
            message_api::V1MessageGetParams {
                app_id,
                msg_id,
                with_content: None,
            },
        )
        .await?)
    }

    pub async fn expunge_content(&self, app_id: String, msg_id: String) -> Result<()> {
        Ok(message_api::v1_message_expunge_content(
            self.cfg,
            message_api::V1MessageExpungeContentParams { msg_id, app_id },
        )
        .await?)
    }
}

#[derive(Default)]
pub struct MessageAttemptListOptions {
    pub iterator: Option<String>,
    pub limit: Option<i32>,
    pub event_types: Option<Vec<String>>,
    // FIXME: make before and after actual dates
    /// RFC3339 date string
    pub before: Option<String>,
    /// RFC3339 date string
    pub after: Option<String>,
    pub channel: Option<String>,
    pub status: Option<MessageStatus>,
    pub status_code_class: Option<StatusCodeClass>,
    pub with_content: Option<bool>,
    pub endpoint_id: Option<String>,
}

pub struct MessageAttempt<'a> {
    cfg: &'a Configuration,
}

impl<'a> MessageAttempt<'a> {
    fn new(cfg: &'a Configuration) -> Self {
        Self { cfg }
    }

    pub async fn list_by_msg(
        &self,
        app_id: String,
        msg_id: String,
        options: Option<MessageAttemptListOptions>,
    ) -> Result<ListResponseMessageAttemptOut> {
        let MessageAttemptListOptions {
            iterator,
            limit,
            event_types,
            before,
            after,
            channel,
            status,
            status_code_class,
            endpoint_id,
            with_content: _,
        } = options.unwrap_or_default();
        Ok(message_attempt_api::v1_message_attempt_list_by_msg(
            self.cfg,
            message_attempt_api::V1MessageAttemptListByMsgParams {
                app_id,
                msg_id,
                iterator,
                limit,
                event_types,
                before,
                after,
                channel,
                status,
                status_code_class,
                endpoint_id,
            },
        )
        .await?)
    }

    pub async fn list_by_endpoint(
        &self,
        app_id: String,
        endpoint_id: String,
        options: Option<MessageAttemptListOptions>,
    ) -> Result<ListResponseMessageAttemptOut> {
        let MessageAttemptListOptions {
            iterator,
            limit,
            event_types,
            before,
            after,
            channel,
            status,
            status_code_class,
            endpoint_id: _,
            with_content: _,
        } = options.unwrap_or_default();
        Ok(message_attempt_api::v1_message_attempt_list_by_endpoint(
            self.cfg,
            message_attempt_api::V1MessageAttemptListByEndpointParams {
                app_id,
                endpoint_id,
                iterator,
                limit,
                event_types,
                before,
                after,
                channel,
                status,
                status_code_class,
            },
        )
        .await?)
    }

    pub async fn list_attempted_messages(
        &self,
        app_id: String,
        endpoint_id: String,
        options: Option<MessageAttemptListOptions>,
    ) -> Result<ListResponseEndpointMessageOut> {
        let MessageAttemptListOptions {
            iterator,
            limit,
            event_types: _,
            before,
            after,
            channel,
            status,
            status_code_class: _,
            with_content,
            endpoint_id: _,
        } = options.unwrap_or_default();
        Ok(
            message_attempt_api::v1_message_attempt_list_attempted_messages(
                self.cfg,
                message_attempt_api::V1MessageAttemptListAttemptedMessagesParams {
                    app_id,
                    endpoint_id,
                    iterator,
                    limit,
                    before,
                    after,
                    channel,
                    status,
                    with_content,
                },
            )
            .await?,
        )
    }

    pub async fn list_attempted_destinations(
        &self,
        app_id: String,
        msg_id: String,
        options: Option<ListOptions>,
    ) -> Result<ListResponseMessageEndpointOut> {
        let ListOptions { iterator, limit } = options.unwrap_or_default();
        Ok(
            message_attempt_api::v1_message_attempt_list_attempted_destinations(
                self.cfg,
                message_attempt_api::V1MessageAttemptListAttemptedDestinationsParams {
                    app_id,
                    msg_id,
                    iterator,
                    limit,
                },
            )
            .await?,
        )
    }

    pub async fn list_attempts_for_endpoint(
        &self,
        app_id: String,
        msg_id: String,
        endpoint_id: String,
        options: Option<MessageAttemptListOptions>,
    ) -> Result<ListResponseMessageAttemptEndpointOut> {
        let MessageAttemptListOptions {
            iterator,
            limit,
            event_types,
            before,
            after,
            channel,
            status,
            status_code_class: _,
            endpoint_id: _,
            with_content: _,
        } = options.unwrap_or_default();
        Ok(
            message_attempt_api::v1_message_attempt_list_by_endpoint_deprecated(
                self.cfg,
                message_attempt_api::V1MessageAttemptListByEndpointDeprecatedParams {
                    app_id,
                    endpoint_id,
                    msg_id,
                    iterator,
                    limit,
                    event_types,
                    before,
                    after,
                    channel,
                    status,
                },
            )
            .await?,
        )
    }

    pub async fn get(
        &self,
        app_id: String,
        msg_id: String,
        attempt_id: String,
    ) -> Result<MessageAttemptOut> {
        Ok(message_attempt_api::v1_message_attempt_get(
            self.cfg,
            message_attempt_api::V1MessageAttemptGetParams {
                app_id,
                msg_id,
                attempt_id,
            },
        )
        .await?)
    }

    pub async fn resend(&self, app_id: String, msg_id: String, endpoint_id: String) -> Result<()> {
        Ok(message_attempt_api::v1_message_attempt_resend(
            self.cfg,
            message_attempt_api::V1MessageAttemptResendParams {
                app_id,
                msg_id,
                endpoint_id,
                idempotency_key: None,
            },
        )
        .await?)
    }

    pub async fn expunge_content(
        &self,
        app_id: String,
        msg_id: String,
        attempt_id: String,
    ) -> Result<()> {
        Ok(message_attempt_api::v1_message_attempt_expunge_content(
            self.cfg,
            message_attempt_api::V1MessageAttemptExpungeContentParams {
                app_id,
                msg_id,
                attempt_id,
            },
        )
        .await?)
    }
}

#[derive(Default)]
pub struct BackgroundTaskListOptions {
    pub iterator: Option<String>,
    pub limit: Option<i32>,
    pub order: Option<Ordering>,
    pub status: Option<BackgroundTaskStatus>,
    pub task: Option<BackgroundTaskType>,
}

pub struct BackgroundTask<'a> {
    cfg: &'a Configuration,
}

impl<'a> BackgroundTask<'a> {
    pub async fn list(
        &self,
        options: Option<BackgroundTaskListOptions>,
    ) -> Result<ListResponseBackgroundTaskOut> {
        let BackgroundTaskListOptions {
            iterator,
            limit,
            order,
            status,
            task,
        } = options.unwrap_or_default();
        Ok(background_tasks_api::list_background_tasks(
            self.cfg,
            background_tasks_api::ListBackgroundTasksParams {
                status,
                task,
                limit,
                iterator,
                order,
            },
        )
        .await?)
    }

    pub async fn get(&self, task_id: String) -> Result<BackgroundTaskOut> {
        Ok(background_tasks_api::get_background_task(
            self.cfg,
            background_tasks_api::GetBackgroundTaskParams { task_id },
        )
        .await?)
    }
}
