import * as cdk from "@aws-cdk/core";
import * as lambda from "@aws-cdk/aws-lambda";
import * as iam from "@aws-cdk/aws-iam";
import * as path from "path";
import * as apigw from "@aws-cdk/aws-apigateway";
import * as cw from "@aws-cdk/aws-cloudwatch";
import * as ssm from "@aws-cdk/aws-ssm";
import * as logs from "@aws-cdk/aws-logs";
import { TokensDbStack } from "./tokens-db-stack";
import { LogsWidgetStack } from "./logs-widget-stack";
import { ApiHandlerStack } from "./api-handler-stack";

export interface WagonApiStackProps extends cdk.StackProps {
  dashboard: cw.Dashboard;
  openid_config_uri: string;
  openid_aud: string;
  tokens_db_stack: TokensDbStack;
}

export class WagonApiStack extends cdk.Stack {
  api: apigw.RestApi;

  constructor(scope: cdk.Construct, id: string, props: WagonApiStackProps) {
    super(scope, id, props);

    const logGroup = new logs.LogGroup(this, id + "ApiLogs");

    const handlerStack = new ApiHandlerStack(this, 'ApiHandler', {
      openid_aud: props.openid_aud,
      openid_config_uri: props.openid_config_uri,
      token_db_stack: props.tokens_db_stack,
    });

    const authorizerRole = new iam.Role(this, "AuthorizerFunctionRole", {
      assumedBy: new iam.ServicePrincipal("lambda.amazonaws.com"),
    });

    authorizerRole.addManagedPolicy(
      iam.ManagedPolicy.fromAwsManagedPolicyName(
          "service-role/AWSLambdaBasicExecutionRole"
      )
    );

    props.tokens_db_stack.tokensTable.grantReadWriteData(authorizerRole);

    const authorizerHandler = new lambda.Function(this, "AuthorizerFunction", {
        runtime: lambda.Runtime.PROVIDED_AL2,
        handler: "unused",
        code: lambda.Code.fromAsset(path.join("..", "target", "lambda-authorizer.zip")),
        memorySize: 128,
        role: authorizerRole,
        timeout: cdk.Duration.seconds(2),
        environment: {
            RUST_LOG: 'info,authorizer=debug',
            OPENID_CONFIGURATION_URI: props.openid_config_uri,
            OPENID_AUD: props.openid_aud,
            TOKENS_TABLE: props.tokens_db_stack.tokensTable.tableName,
            TOKENS_TABLE_TOKENS_INDEX: props.tokens_db_stack.tokensIndexName,
        },
    });

    const authorizer = new apigw.TokenAuthorizer(this, "Authorizer", {
        handler: authorizerHandler,
    });

    const api = new apigw.RestApi(this, id + "RestApi", {
      endpointTypes: [apigw.EndpointType.REGIONAL],
      deployOptions: {
        loggingLevel: apigw.MethodLoggingLevel.INFO,
        accessLogDestination: new apigw.LogGroupLogDestination(logGroup),
        accessLogFormat: apigw.AccessLogFormat.jsonWithStandardFields()
      },
      defaultMethodOptions: {
        authorizer: authorizer,
      },
      defaultIntegration: new apigw.LambdaIntegration(handlerStack.handler, {
        proxy: true,
        //contentHandling: apigw.ContentHandling.CONVERT_TO_TEXT,
        passthroughBehavior: apigw.PassthroughBehavior.WHEN_NO_MATCH,
      }),
      binaryMediaTypes: [
        "application/octet-stream"
      ],
    });

    /*
    GET / => get_root,
        GET /api/token => get_token,
        POST /api/token => create_token,
        PUT /api/v1/crates/new => new_crate,
        GET /api/v1/crates => search_crates,
        GET /api/v1/crates/{library: String}/{version: String}/download => download_crate,
        DELETE /api/v1/crates/{library: String}/{version: String}/yank => yank_crate,
        PUT /api/v1/crates/{library: String}/{version: String}/unyank => unyank_crate,
        GET /api/v1/crates/{library: String}/owners => get_crate_owners,
        PUT /api/v1/crates/{library: String}/owners => add_crate_owner,
        DELETE /api/v1/crates/{library: String}/owners => remove_crate_owner,
    */
    const api_resource = api.root.addResource('api');

    const api_token_resource = api_resource.addResource('token');
    api_token_resource.addMethod('GET');
    api_token_resource.addMethod('POST');

    const api_v1_resource = api_resource.addResource('v1');
    
    const api_v1_crates_resource = api_v1_resource.addResource('crates');
    api_v1_crates_resource.addMethod('GET');

    const api_v1_crates_resource_new = api_v1_crates_resource.addResource('new');
    api_v1_crates_resource_new.addMethod('PUT', new apigw.LambdaIntegration(handlerStack.handler, {
      contentHandling: apigw.ContentHandling.CONVERT_TO_TEXT,
      requestTemplates: {
        'application/octet-stream': JSON.stringify({ body: '$input.body', contentType: 'octet-stream' }),
        '*/*': JSON.stringify({ body: '$input.body', contentType: 'any' })
      }
    }));

    const api_v1_crates_crate_resource = api_v1_crates_resource.addResource('{crate}');

    const api_v1_crates_crate_owners_resource = api_v1_crates_crate_resource.addResource('owners');
    api_v1_crates_crate_owners_resource.addMethod('GET');
    api_v1_crates_crate_owners_resource.addMethod('PUT');
    api_v1_crates_crate_owners_resource.addMethod('DELETE');

    const api_v1_crates_crate_version_resource = api_v1_crates_crate_resource.addResource('{version}');
    
    const api_v1_crates_crate_version_download_resource = api_v1_crates_crate_version_resource.addResource('download');
    api_v1_crates_crate_version_download_resource.addMethod('GET');

    const api_v1_crates_crate_version_yank_resource = api_v1_crates_crate_version_resource.addResource('yank');
    api_v1_crates_crate_version_download_resource.addMethod('DELETE');
    
    const api_v1_crates_crate_version_unyank_resource = api_v1_crates_crate_version_resource.addResource('unyank');
    api_v1_crates_crate_version_download_resource.addMethod('PUT');


    new cdk.CfnOutput(this, 'WagonApiDomainNameOutput', {
      value: `${api.restApiId}.execute-api.${this.region}.amazonaws.com`,
      exportName: "WagonApiDomainName",
    });

    new ssm.StringParameter(this, "WagonApiDomainNameParameter", {
      stringValue: `${api.restApiId}.execute-api.${this.region}.amazonaws.com`,
      parameterName: "wagon-api-domain-name",
    });

    new cdk.CfnOutput(this, 'WagonApiPathOutput', {
      value: `/${api.deploymentStage.stageName}`,
      exportName: "WagonApiPath",
    });

    new ssm.StringParameter(this, "WagonApiPathParameter", {
      stringValue: `/${api.deploymentStage.stageName}`,
      parameterName: "wagon-api-path",
    });

    new LogsWidgetStack(this, "WagonLogsWidget", {
      dashboard: props.dashboard,
      logGroupNames: [
          handlerStack.handler.logGroup.logGroupName,
          authorizerHandler.logGroup.logGroupName,
          logGroup.logGroupName,
      ]
    });
  }
}
