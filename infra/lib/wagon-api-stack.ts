import * as cdk from "@aws-cdk/core";
import * as lambda from "@aws-cdk/aws-lambda";
import * as iam from "@aws-cdk/aws-iam";
import * as path from "path";
import * as apigw from "@aws-cdk/aws-apigateway";

export class WagonApiStack extends cdk.Stack {
  constructor(scope: cdk.Construct, id: string, props?: cdk.StackProps) {
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
        RUST_LOG: 'debug'
      }
    });

    const api = new apigw.LambdaRestApi(this, id + "RestApi", {
      handler: handler
    });

    new cdk.CfnOutput(this, 'RegistryApiUrl', {
      value: `https://${api.restApiId}.execute-api.${this.region}.amazonaws.com/${api.deploymentStage.stageName}`,
    });
  }
}
