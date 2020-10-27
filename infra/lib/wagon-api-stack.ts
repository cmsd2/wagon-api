import * as cdk from "@aws-cdk/core";
import * as lambda from "@aws-cdk/aws-lambda";
import * as iam from "@aws-cdk/aws-iam";
import * as path from "path";
import * as apigw from "@aws-cdk/aws-apigateway";
import * as cw from "@aws-cdk/aws-cloudwatch";
import * as logs from "@aws-cdk/aws-logs";
import { DashboardStack } from "./dashboard-stack";

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
    });

    new cdk.CfnOutput(this, 'WagonApiDomainName', {
      value: `${api.restApiId}.execute-api.${this.region}.amazonaws.com`,
    });

    new cdk.CfnOutput(this, 'WagonApiPath', {
      value: `/${api.deploymentStage.stageName}`,
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
