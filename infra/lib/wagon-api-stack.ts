import * as cdk from "@aws-cdk/core";
import * as lambda from "@aws-cdk/aws-lambda";
import * as iam from "@aws-cdk/aws-iam";
import * as path from "path";

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

    new lambda.Function(this, "Function", {
      runtime: lambda.Runtime.PROVIDED,
      handler: "unused",
      code: lambda.Code.fromAsset(path.join("..", "dist", "api", "lambda.zip")),
      memorySize: 128,
      role: lambdaRole,
      timeout: cdk.Duration.seconds(2),
    });
  }
}
