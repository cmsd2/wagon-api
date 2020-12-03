import * as cdk from "@aws-cdk/core";
import * as lambda from "@aws-cdk/aws-lambda";
import * as iam from "@aws-cdk/aws-iam";
import * as path from "path";
import * as apigw from "@aws-cdk/aws-apigateway";
import * as cw from "@aws-cdk/aws-cloudwatch";
import * as ssm from "@aws-cdk/aws-ssm";
import * as logs from "@aws-cdk/aws-logs";
import * as ddb from "@aws-cdk/aws-dynamodb";
import { DashboardStack } from "./dashboard-stack";
import { WagonApiStack } from "./wagon-api-stack";

export interface TokensApiStackProps extends cdk.StackProps {
  dashboard: cw.Dashboard,
  api: WagonApiStack,
}

export class TokensApiStack extends cdk.Stack {
    tokensTable: ddb.Table;
    handler: lambda.Function;

    constructor(scope: cdk.Construct, id: string, props: TokensApiStackProps) {
        super(scope, id, props);

        this.tokensTable = new ddb.Table(this, 'Tokens', {
            partitionKey: {
                name: 'user_id', type: ddb.AttributeType.STRING
            },
            billingMode: ddb.BillingMode.PAY_PER_REQUEST,
            encryption: ddb.TableEncryption.DEFAULT,
        });

        const lambdaRole = new iam.Role(this, "FunctionRole", {
            assumedBy: new iam.ServicePrincipal("lambda.amazonaws.com"),
          });
      
        lambdaRole.addManagedPolicy(
            iam.ManagedPolicy.fromAwsManagedPolicyName(
                "service-role/AWSLambdaBasicExecutionRole"
            )
        );

        lambdaRole.addToPolicy(new iam.PolicyStatement({
            resources: ['*'],
            actions: ['kms:GenerateRandom']
        }));

        lambdaRole.addToPolicy(new iam.PolicyStatement({
            resources: [this.tokensTable.tableArn],
            actions: ['dynamodb:GetItem','dynamodb:PutItem','dynamodb:UpdateItem']
        }));

        this.handler = new lambda.Function(this, "Function", {
            runtime: lambda.Runtime.PROVIDED_AL2,
            handler: "unused",
            code: lambda.Code.fromAsset(path.join("..", "target", "lambda-api.zip")),
            memorySize: 128,
            role: lambdaRole,
            timeout: cdk.Duration.seconds(2),
            environment: {
                RUST_LOG: 'info,api=debug'
            },
        });

        props.api.apiResource.addResource('token').addMethod('ANY', new apigw.LambdaIntegration(this.handler, {
            proxy: true,
        }), {
            authorizer: props.api.jwtAuthorizer,
        });
    }
}
