import * as cdk from "@aws-cdk/core";
import * as lambda from "@aws-cdk/aws-lambda";
import * as iam from "@aws-cdk/aws-iam";
import * as path from "path";
import * as apigw from "@aws-cdk/aws-apigateway";
import * as cw from "@aws-cdk/aws-cloudwatch";
import * as ssm from "@aws-cdk/aws-ssm";
import * as logs from "@aws-cdk/aws-logs";
import { DashboardStack } from "./dashboard-stack";
import { SSL_OP_MICROSOFT_BIG_SSLV3_BUFFER } from "constants";

export interface WagonApiStackProps extends cdk.StackProps {
  dashboard: cw.Dashboard,
}

export class WagonApiStack extends cdk.Stack {
  constructor(scope: cdk.Construct, id: string, props: WagonApiStackProps) {
    super(scope, id, props);

    const lambdaRole = new iam.Role(this, "FunctionRole", {
      assumedBy: new iam.ServicePrincipal("lambda.amazonaws.com"),
    });

    lambdaRole.addManagedPolicy(
      iam.ManagedPolicy.fromAwsManagedPolicyName(
        "service-role/AWSLambdaBasicExecutionRole"
      )
    );

    const handler = new lambda.Function(this, "Function", {
      runtime: lambda.Runtime.PROVIDED_AL2,
      handler: "unused",
      code: lambda.Code.fromAsset(path.join("..", "target", "lambda.zip")),
      memorySize: 128,
      role: lambdaRole,
      timeout: cdk.Duration.seconds(2),
      environment: {
        RUST_LOG: 'info,api=debug'
      },
    });

    const api = new apigw.LambdaRestApi(this, id + "RestApi", {
      handler: handler,
      endpointTypes: [apigw.EndpointType.REGIONAL],
      deployOptions: {
        loggingLevel: apigw.MethodLoggingLevel.INFO,
      }
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

    props.dashboard.addWidgets(new cw.LogQueryWidget({
      logGroupNames: [handler.logGroup.logGroupName],
      title: "Wagon Api Logs",
      width: 24,
      queryLines: [
        "fields @timestamp, @message",
        "sort @timestamp desc",
        "limit 200",
      ]
    }));
  }
}
