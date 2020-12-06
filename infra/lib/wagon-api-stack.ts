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

    const api = new apigw.LambdaRestApi(this, id + "RestApi", {
      endpointTypes: [apigw.EndpointType.REGIONAL],
      deployOptions: {
        loggingLevel: apigw.MethodLoggingLevel.INFO,
        accessLogDestination: new apigw.LogGroupLogDestination(logGroup),
        accessLogFormat: apigw.AccessLogFormat.jsonWithStandardFields()
      },
      handler: handlerStack.handler,
      defaultMethodOptions: {
        authorizer: authorizer,
      },
      proxy: true,
    });

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
