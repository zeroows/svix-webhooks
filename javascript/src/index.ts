import {
  ServerConfiguration,
  HttpBearerConfiguration,
  Configuration,
  createConfiguration,
  ApplicationApi,
  ApplicationOut,
  ListResponseApplicationOut,
  ApplicationIn,
  EndpointApi,
  ListResponseEndpointOut,
  EndpointOut,
  EndpointIn,
  EndpointUpdate,
  EndpointSecretOut,
  EndpointSecretRotateIn,
  EndpointTransformationIn,
  EndpointTransformationOut,
  EndpointHeadersIn,
  EndpointHeadersPatchIn,
  EndpointHeadersOut,
  EndpointStats,
  EventExampleIn,
  RecoverIn,
  ReplayIn,
  IntegrationApi,
  ListResponseIntegrationOut,
  IntegrationOut,
  IntegrationIn,
  IntegrationUpdate,
  IntegrationKeyOut,
  MessageApi,
  MessageOut,
  MessageIn,
  ListResponseMessageOut,
  AuthenticationApi,
  DashboardAccessOut,
  MessageAttemptApi,
  ListResponseEndpointMessageOut,
  ListResponseMessageEndpointOut,
  ListResponseMessageAttemptEndpointOut,
  ListResponseMessageAttemptOut,
  MessageAttemptOut,
  MessageStatus,
  EventTypeApi,
  ListResponseEventTypeOut,
  EventTypeIn,
  EventTypeOut,
  EventTypeUpdate,
  StatusCodeClass,
  Middleware,
  RequestContext,
  ResponseContext,
  AppPortalAccessOut,
  AppPortalAccessIn,
  Ordering,
  BackgroundTaskStatus,
  BackgroundTaskType,
  BackgroundTaskOut,
  ListResponseBackgroundTaskOut,
  BackgroundTasksApi,
} from "./openapi/index";
export * from "./openapi/models/all";
export * from "./openapi/apis/exception";
import { timingSafeEqual } from "./timing_safe_equal";
import * as base64 from "@stablelib/base64";
import * as sha256 from "fast-sha256";

const WEBHOOK_TOLERANCE_IN_SECONDS = 5 * 60; // 5 minutes
const VERSION = "1.5.2";

class UserAgentMiddleware implements Middleware {
  public pre(context: RequestContext): Promise<RequestContext> {
    context.setHeaderParam("User-Agent", `svix-libs/${VERSION}/javascript`);
    return Promise.resolve(context);
  }

  public post(context: ResponseContext): Promise<ResponseContext> {
    return Promise.resolve(context);
  }
}

export interface SvixOptions {
  debug?: boolean;
  serverUrl?: string;
}

const REGIONS = [
  { region: "us", url: "https://api.us.svix.com" },
  { region: "eu", url: "https://api.eu.svix.com" },
  { region: "in", url: "https://api.in.svix.com" },
];

export class Svix {
  public readonly _configuration: Configuration;
  public readonly authentication: Authentication;
  public readonly application: Application;
  public readonly endpoint: Endpoint;
  public readonly eventType: EventType;
  public readonly integration: Integration;
  public readonly message: Message;
  public readonly messageAttempt: MessageAttempt;
  public readonly backgroundTask: BackgroundTask;

  public constructor(token: string, options: SvixOptions = {}) {
    const regionalUrl = REGIONS.find((x) => x.region === token.split(".")[1])?.url;
    const baseUrl: string = options.serverUrl ?? regionalUrl ?? "https://api.svix.com";

    const baseServer = new ServerConfiguration<any>(baseUrl, {});

    const bearerConfiguration: HttpBearerConfiguration = {
      tokenProvider: {
        getToken: () => token,
      },
    };
    const config = createConfiguration({
      baseServer,
      promiseMiddleware: [new UserAgentMiddleware()],
      authMethods: {
        HTTPBearer: bearerConfiguration,
      },
    });

    this._configuration = config;
    this.authentication = new Authentication(config);
    this.application = new Application(config);
    this.endpoint = new Endpoint(config);
    this.eventType = new EventType(config);
    this.integration = new Integration(config);
    this.message = new Message(config);
    this.messageAttempt = new MessageAttempt(config);
    this.backgroundTask = new BackgroundTask(config);
  }
}
export interface PostOptions {
  idempotencyKey?: string;
}
class Authentication {
  private readonly api: AuthenticationApi;

  public constructor(config: Configuration) {
    this.api = new AuthenticationApi(config);
  }

  public appPortalAccess(
    appId: string,
    appPortalAccessIn: AppPortalAccessIn,
    options?: PostOptions
  ): Promise<AppPortalAccessOut> {
    return this.api.v1AuthenticationAppPortalAccess({
      appId,
      appPortalAccessIn,
      ...options,
    });
  }

  public dashboardAccess(
    appId: string,
    options?: PostOptions
  ): Promise<DashboardAccessOut> {
    return this.api.v1AuthenticationDashboardAccess({
      appId,
      ...options,
    });
  }

  public logout(options?: PostOptions): Promise<void> {
    return this.api.v1AuthenticationLogout({ ...options });
  }
}

interface ListOptions {
  iterator?: string;
  limit?: number;
}

export interface ApplicationListOptions extends ListOptions {
  order?: Ordering;
}

export interface EndpointListOptions extends ListOptions {
  order?: Ordering;
}

export interface EndpointStatsOptions {
  since?: Date;
  until?: Date;
}

export type IntegrationListOptions = ListOptions;

export interface EventTypeListOptions extends ListOptions {
  withContent?: boolean;
  includeArchived?: boolean;
}

export interface MessageListOptions extends ListOptions {
  eventTypes?: string[];
  before?: Date;
  after?: Date;
  channel?: string;
  withContent?: boolean;
}

export interface MessageAttemptListOptions extends ListOptions {
  status?: MessageStatus;
  eventTypes?: string[];
  before?: Date;
  after?: Date;
  statusCodeClass?: StatusCodeClass;
  channel?: string;
}

export interface BackgroundTaskListOptions extends ListOptions {
  status?: BackgroundTaskStatus;
  task?: BackgroundTaskType;
}

class Application {
  private readonly api: ApplicationApi;

  public constructor(config: Configuration) {
    this.api = new ApplicationApi(config);
  }

  public list(options?: ApplicationListOptions): Promise<ListResponseApplicationOut> {
    return this.api.v1ApplicationList({ ...options });
  }

  public create(
    applicationIn: ApplicationIn,
    options?: PostOptions
  ): Promise<ApplicationOut> {
    return this.api.v1ApplicationCreate({ applicationIn, ...options });
  }

  public getOrCreate(
    applicationIn: ApplicationIn,
    options?: PostOptions
  ): Promise<ApplicationOut> {
    return this.api.v1ApplicationCreate({
      applicationIn,
      ...options,
      getIfExists: true,
    });
  }

  public get(appId: string): Promise<ApplicationOut> {
    return this.api.v1ApplicationGet({ appId });
  }

  public update(appId: string, applicationIn: ApplicationIn): Promise<ApplicationOut> {
    return this.api.v1ApplicationUpdate({ appId, applicationIn });
  }

  public delete(appId: string): Promise<void> {
    return this.api.v1ApplicationDelete({ appId });
  }
}

class Endpoint {
  private readonly api: EndpointApi;

  public constructor(config: Configuration) {
    this.api = new EndpointApi(config);
  }

  public list(
    appId: string,
    options?: EndpointListOptions
  ): Promise<ListResponseEndpointOut> {
    return this.api.v1EndpointList({ appId, ...options });
  }

  public create(
    appId: string,
    endpointIn: EndpointIn,
    options?: PostOptions
  ): Promise<EndpointOut> {
    return this.api.v1EndpointCreate({
      appId,
      endpointIn,
      ...options,
    });
  }

  public get(appId: string, endpointId: string): Promise<EndpointOut> {
    return this.api.v1EndpointGet({ endpointId, appId });
  }

  public update(
    appId: string,
    endpointId: string,
    endpointUpdate: EndpointUpdate
  ): Promise<EndpointOut> {
    return this.api.v1EndpointUpdate({
      appId,
      endpointId,
      endpointUpdate,
    });
  }

  public delete(appId: string, endpointId: string): Promise<void> {
    return this.api.v1EndpointDelete({
      endpointId,
      appId,
    });
  }

  public getSecret(appId: string, endpointId: string): Promise<EndpointSecretOut> {
    return this.api.v1EndpointGetSecret({
      endpointId,
      appId,
    });
  }

  public rotateSecret(
    appId: string,
    endpointId: string,
    endpointSecretRotateIn: EndpointSecretRotateIn,
    options?: PostOptions
  ): Promise<void> {
    return this.api.v1EndpointRotateSecret({
      endpointId,
      appId,
      endpointSecretRotateIn,
      ...options,
    });
  }

  public recover(
    appId: string,
    endpointId: string,
    recoverIn: RecoverIn,
    options?: PostOptions
  ): Promise<void> {
    return this.api
      .v1EndpointRecover({
        appId,
        endpointId,
        recoverIn,
        ...options,
      })
      .then(() => Promise.resolve());
  }

  public replayMissing(
    appId: string,
    endpointId: string,
    replayIn: ReplayIn,
    options?: PostOptions
  ): Promise<void> {
    return this.api
      .v1EndpointReplay({
        appId,
        endpointId,
        replayIn,
        ...options,
      })
      .then(() => Promise.resolve());
  }

  public getHeaders(appId: string, endpointId: string): Promise<EndpointHeadersOut> {
    return this.api.v1EndpointGetHeaders({
      appId,
      endpointId,
    });
  }

  public updateHeaders(
    appId: string,
    endpointId: string,
    endpointHeadersIn: EndpointHeadersIn
  ): Promise<void> {
    return this.api.v1EndpointUpdateHeaders({
      appId,
      endpointId,
      endpointHeadersIn,
    });
  }

  public patchHeaders(
    appId: string,
    endpointId: string,
    endpointHeadersPatchIn: EndpointHeadersPatchIn
  ): Promise<void> {
    return this.api.v1EndpointPatchHeaders({
      appId,
      endpointId,
      endpointHeadersPatchIn,
    });
  }

  public getStats(appId: string, endpointId: string, options?: EndpointStatsOptions): Promise<EndpointStats> {
    return this.api.v1EndpointGetStats({
      appId,
      endpointId,
      ...options
    });
  }

  public transformationGet(
    appId: string,
    endpointId: string
  ): Promise<EndpointTransformationOut> {
    return this.api.v1EndpointTransformationGet(
      { endpointId, appId }
    );
  }

  public transformationPartialUpdate(
    appId: string,
    endpointId: string,
    endpointTransformationIn: EndpointTransformationIn
  ): Promise<void> {
    return this.api.v1EndpointTransformationPartialUpdate(
      { appId, endpointId, endpointTransformationIn }
    );
  }

  public sendExample(
    appId: string,
    endpointId: string,
    eventExampleIn: EventExampleIn,
    options?: PostOptions,
  ): Promise<MessageOut> {
    return this.api.v1EndpointSendExample(
      { appId, endpointId, eventExampleIn, ...options }
    );
  }
}

class EventType {
  private readonly api: EventTypeApi;

  public constructor(config: Configuration) {
    this.api = new EventTypeApi(config);
  }

  public list(options?: EventTypeListOptions): Promise<ListResponseEventTypeOut> {
    return this.api.v1EventTypeList({ ...options });
  }

  public get(eventTypeName: string): Promise<EventTypeOut> {
    return this.api.v1EventTypeGet({ eventTypeName });
  }

  public create(eventTypeIn: EventTypeIn, options?: PostOptions): Promise<EventTypeOut> {
    return this.api.v1EventTypeCreate({ eventTypeIn, ...options });
  }

  public update(
    eventTypeName: string,
    eventTypeUpdate: EventTypeUpdate
  ): Promise<EventTypeOut> {
    return this.api.v1EventTypeUpdate({
      eventTypeName,
      eventTypeUpdate,
    });
  }

  public delete(eventTypeName: string): Promise<void> {
    return this.api.v1EventTypeDelete({ eventTypeName });
  }
}

class Integration {
  private readonly api: IntegrationApi;

  public constructor(config: Configuration) {
    this.api = new IntegrationApi(config);
  }

  public list(
    appId: string,
    options?: IntegrationListOptions
  ): Promise<ListResponseIntegrationOut> {
    return this.api.v1IntegrationList({ appId, ...options });
  }

  public create(
    appId: string,
    integrationIn: IntegrationIn,
    options?: PostOptions
  ): Promise<IntegrationOut> {
    return this.api.v1IntegrationCreate({
      appId,
      integrationIn,
      ...options,
    });
  }

  public get(appId: string, integId: string): Promise<IntegrationOut> {
    return this.api.v1IntegrationGet({ integId, appId });
  }

  public update(
    appId: string,
    integId: string,
    integrationUpdate: IntegrationUpdate
  ): Promise<IntegrationOut> {
    return this.api.v1IntegrationUpdate({
      appId,
      integId,
      integrationUpdate,
    });
  }

  public delete(appId: string, integId: string): Promise<void> {
    return this.api.v1IntegrationDelete({
      integId,
      appId,
    });
  }

  public getKey(appId: string, integId: string): Promise<IntegrationKeyOut> {
    return this.api.v1IntegrationGetKey({
      integId,
      appId,
    });
  }

  public rotateKey(
    appId: string,
    integId: string,
    options?: PostOptions
  ): Promise<IntegrationKeyOut> {
    return this.api.v1IntegrationRotateKey({
      integId,
      appId,
      ...options,
    });
  }
}

class Message {
  private readonly api: MessageApi;

  public constructor(config: Configuration) {
    this.api = new MessageApi(config);
  }

  public list(
    appId: string,
    options?: MessageListOptions
  ): Promise<ListResponseMessageOut> {
    return this.api.v1MessageList({ appId, ...options });
  }

  public create(
    appId: string,
    messageIn: MessageIn,
    options?: PostOptions
  ): Promise<MessageOut> {
    return this.api.v1MessageCreate({ appId, messageIn, ...options });
  }

  public get(appId: string, msgId: string): Promise<MessageOut> {
    return this.api.v1MessageGet({ msgId, appId });
  }

  public expungeContent(appId: string, msgId: string): Promise<void> {
    return this.api.v1MessageExpungeContent({ appId, msgId });
  }
}

class MessageAttempt {
  private readonly api: MessageAttemptApi;

  public constructor(config: Configuration) {
    this.api = new MessageAttemptApi(config);
  }

  /**
   * @deprecated Since version 0.48.0. Use listByMsg or listByEndpoint instead.
   */
  public list(
    appId: string,
    msgId: string,
    options?: MessageAttemptListOptions
  ): Promise<ListResponseMessageAttemptOut> {
    return this.listByMsg(appId, msgId, options);
  }

  public listByMsg(
    appId: string,
    msgId: string,
    options?: MessageAttemptListOptions
  ): Promise<ListResponseMessageAttemptOut> {
    return this.api.v1MessageAttemptListByMsg({
      appId,
      msgId,
      ...options,
    });
  }

  public listByEndpoint(
    appId: string,
    endpointId: string,
    options?: MessageAttemptListOptions
  ): Promise<ListResponseMessageAttemptOut> {
    return this.api.v1MessageAttemptListByEndpoint({
      appId,
      endpointId,
      ...options,
    });
  }

  public get(
    appId: string,
    msgId: string,
    attemptId: string
  ): Promise<MessageAttemptOut> {
    return this.api.v1MessageAttemptGet({
      attemptId,
      msgId,
      appId,
    });
  }

  public resend(
    appId: string,
    msgId: string,
    endpointId: string,
    options?: PostOptions
  ): Promise<void> {
    return this.api.v1MessageAttemptResend({
      endpointId,
      msgId,
      appId,
      ...options,
    });
  }

  public listAttemptedMessages(
    appId: string,
    endpointId: string,
    options?: MessageAttemptListOptions
  ): Promise<ListResponseEndpointMessageOut> {
    return this.api.v1MessageAttemptListAttemptedMessages({
      appId,
      endpointId,
      ...options,
    });
  }

  public listAttemptedDestinations(
    appId: string,
    msgId: string,
    options?: MessageAttemptListOptions
  ): Promise<ListResponseMessageEndpointOut> {
    return this.api.v1MessageAttemptListAttemptedDestinations({
      appId,
      msgId,
      ...options,
    });
  }

  public listAttemptsForEndpoint(
    appId: string,
    msgId: string,
    endpointId: string,
    options?: MessageAttemptListOptions
  ): Promise<ListResponseMessageAttemptEndpointOut> {
    return this.api.v1MessageAttemptListByEndpointDeprecated(
      { appId, msgId, endpointId, ...options }
    );
  }

  public expungeContent(
    appId: string,
    msgId: string,
    attemptId: string
  ): Promise<void> {
    return this.api.v1MessageAttemptExpungeContent({
      appId, msgId, attemptId
    })
  }
}

class BackgroundTask {
  private readonly api: BackgroundTasksApi;

  public constructor(config: Configuration) {
    this.api = new BackgroundTasksApi(config);
  }

  public listByEndpoint(
    options?: BackgroundTaskListOptions
  ): Promise<ListResponseBackgroundTaskOut> {
    return this.api.listBackgroundTasks({
      ...options,
    });
  }

  public get(
    taskId: string,
  ): Promise<BackgroundTaskOut> {
    return this.api.getBackgroundTask({
      taskId
    });
  }

}

class ExtendableError extends Error {
  constructor(message: any) {
    super(message);
    Object.setPrototypeOf(this, ExtendableError.prototype);
    this.name = "ExtendableError";
    this.stack = new Error(message).stack;
  }
}

export class WebhookVerificationError extends ExtendableError {
  constructor(message: string) {
    super(message);
    Object.setPrototypeOf(this, WebhookVerificationError.prototype);
    this.name = "WebhookVerificationError";
  }
}

export interface WebhookRequiredHeaders {
  "svix-id": string;
  "svix-timestamp": string;
  "svix-signature": string;
}

export interface WebhookUnbrandedRequiredHeaders {
  "webhook-id": string;
  "webhook-timestamp": string;
  "webhook-signature": string;
}

export interface WebhookOptions {
  format?: "raw";
}

export class Webhook {
  private static prefix = "whsec_";
  private readonly key: Uint8Array;

  constructor(secret: string | Uint8Array, options?: WebhookOptions) {
    if (!secret) {
      throw new Error("Secret can't be empty.");
    }
    if (options?.format === "raw") {
      if (secret instanceof Uint8Array) {
        this.key = secret;
      } else {
        this.key = Uint8Array.from(secret, (c) => c.charCodeAt(0));
      }
    } else {
      if (typeof secret !== "string") {
        throw new Error("Expected secret to be of type string");
      }
      if (secret.startsWith(Webhook.prefix)) {
        secret = secret.substring(Webhook.prefix.length);
      }
      this.key = base64.decode(secret);
    }
  }

  public verify(
    payload: string | Buffer,
    headers_:
      | WebhookRequiredHeaders
      | WebhookUnbrandedRequiredHeaders
      | Record<string, string>
  ): unknown {
    const headers: Record<string, string> = {};
    for (const key of Object.keys(headers_)) {
      headers[key.toLowerCase()] = (headers_ as Record<string, string>)[key];
    }

    let msgId = headers["svix-id"];
    let msgSignature = headers["svix-signature"];
    let msgTimestamp = headers["svix-timestamp"];

    if (!msgSignature || !msgId || !msgTimestamp) {
      msgId = headers["webhook-id"];
      msgSignature = headers["webhook-signature"];
      msgTimestamp = headers["webhook-timestamp"];

      if (!msgSignature || !msgId || !msgTimestamp) {
        throw new WebhookVerificationError("Missing required headers");
      }
    }

    const timestamp = this.verifyTimestamp(msgTimestamp);

    const computedSignature = this.sign(msgId, timestamp, payload);
    const expectedSignature = computedSignature.split(",")[1];

    const passedSignatures = msgSignature.split(" ");

    const encoder = new globalThis.TextEncoder();
    for (const versionedSignature of passedSignatures) {
      const [version, signature] = versionedSignature.split(",");
      if (version !== "v1") {
        continue;
      }

      if (timingSafeEqual(encoder.encode(signature), encoder.encode(expectedSignature))) {
        return JSON.parse(payload.toString());
      }
    }
    throw new WebhookVerificationError("No matching signature found");
  }

  public sign(msgId: string, timestamp: Date, payload: string | Buffer): string {
    if (typeof payload === "string") {
      // Do nothing, already a string
    } else if (payload.constructor.name === "Buffer") {
      payload = payload.toString();
    } else {
      throw new Error("Expected payload to be of type string or Buffer. Please refer to https://docs.svix.com/receiving/verifying-payloads/how for more information.");
    }

    const encoder = new TextEncoder();
    const timestampNumber = Math.floor(timestamp.getTime() / 1000);
    const toSign = encoder.encode(`${msgId}.${timestampNumber}.${payload}`);
    const expectedSignature = base64.encode(sha256.hmac(this.key, toSign));
    return `v1,${expectedSignature}`;
  }

  private verifyTimestamp(timestampHeader: string): Date {
    const now = Math.floor(Date.now() / 1000);
    const timestamp = parseInt(timestampHeader, 10);
    if (isNaN(timestamp)) {
      throw new WebhookVerificationError("Invalid Signature Headers");
    }

    if (now - timestamp > WEBHOOK_TOLERANCE_IN_SECONDS) {
      throw new WebhookVerificationError("Message timestamp too old");
    }
    if (timestamp > now + WEBHOOK_TOLERANCE_IN_SECONDS) {
      throw new WebhookVerificationError("Message timestamp too new");
    }
    return new Date(timestamp * 1000);
  }
}
