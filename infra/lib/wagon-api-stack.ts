import * as cdk from "@aws-cdk/core";
import * as lambda from "@aws-cdk/aws-lambda";
import * as iam from "@aws-cdk/aws-iam";
import * as path from "path";
import * as apigw from "@aws-cdk/aws-apigateway";
import * as cw from "@aws-cdk/aws-cloudwatch";
import * as ssm from "@aws-cdk/aws-ssm";
import * as logs from "@aws-cdk/aws-logs";

export interface WagonApiStackProps extends cdk.StackProps {
  dashboard: cw.Dashboard;
  openid_config_uri: string;
  openid_aud: string;
}

export class WagonApiStack extends cdk.Stack {
  jwtAuthorizerFunction: lambda.Function;
  jwtAuthorizer: apigw.TokenAuthorizer;
  apiResource: apigw.Resource;
  logGroup: logs.LogGroup;

  constructor(scope: cdk.Construct, id: string, props: WagonApiStackProps) {
    super(scope, id, props);

    const authorizerLambdaRole = new iam.Role(this, "AuthorizerFunctionRole", {
      assumedBy: new iam.ServicePrincipal("lambda.amazonaws.com"),
    });

    authorizerLambdaRole.addManagedPolicy(
      iam.ManagedPolicy.fromAwsManagedPolicyName(
        "service-role/AWSLambdaBasicExecutionRole"
      )
    );

    this.jwtAuthorizerFunction = new lambda.Function(this, "JwtAuthorizerFunction", {
      runtime: lambda.Runtime.PROVIDED_AL2,
      handler: "unused",
      code: lambda.Code.fromAsset(path.join("..", "target", "lambda-jwt_authorizer.zip")),
      memorySize: 128,
      role: authorizerLambdaRole,
      timeout: cdk.Duration.seconds(2),
      environment: {
        RUST_LOG: 'info,jwt_authorizer=debug',
        OPENID_CONFIGURATION_URI: props.openid_config_uri,
        OPENID_AUD: props.openid_aud,
      },
    });

    this.jwtAuthorizer = new apigw.TokenAuthorizer(this, 'JwtAuthorizer', {
      handler: this.jwtAuthorizerFunction
    });

    const lambdaRole = new iam.Role(this, "FunctionRole", {
      assumedBy: new iam.ServicePrincipal("lambda.amazonaws.com"),
    });

    lambdaRole.addManagedPolicy(
      iam.ManagedPolicy.fromAwsManagedPolicyName(
        "service-role/AWSLambdaBasicExecutionRole"
      )
    );

    this.logGroup = new logs.LogGroup(this, id + "ApiLogs");

    const api = new apigw.RestApi(this, id + "RestApi", {
      endpointTypes: [apigw.EndpointType.REGIONAL],
      deployOptions: {
        loggingLevel: apigw.MethodLoggingLevel.INFO,
        accessLogDestination: new apigw.LogGroupLogDestination(this.logGroup),
        accessLogFormat: apigw.AccessLogFormat.jsonWithStandardFields()
      },
    });

    this.apiResource = api.root.addResource('api');

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
  }
}
