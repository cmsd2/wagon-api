import { Construct } from 'constructs';
import * as cdk from "aws-cdk-lib";
import * as lambda from "aws-cdk-lib/aws-lambda";
import * as iam from "aws-cdk-lib/aws-iam";
import * as path from "path";
import * as apigw from "aws-cdk-lib/aws-apigateway";
import * as cw from "aws-cdk-lib/aws-cloudwatch";
import * as ssm from "aws-cdk-lib/aws-ssm";
import * as logs from "aws-cdk-lib/aws-logs";
import * as ddb from "aws-cdk-lib/aws-dynamodb";
import { DashboardStack } from "./dashboard-stack";
import { WagonApiStack } from "./wagon-api-stack";
import { TokensDbStack } from "./tokens-db-stack";

export interface ApiHandlerStackProps extends cdk.StackProps {
    token_db_stack: TokensDbStack;
}

export class ApiHandlerStack extends cdk.Stack {
    handler: lambda.Function;
    role: iam.Role;

    constructor(scope: Construct, id: string, props: ApiHandlerStackProps) {
        super(scope, id, props);

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
    
        props.token_db_stack.tokensTable.grantReadWriteData(lambdaRole);
    
        this.handler = new lambda.Function(this, "Function", {
            runtime: lambda.Runtime.PROVIDED_AL2,
            handler: "unused",
            code: lambda.Code.fromAsset(path.join("..", "target", "lambda/api/bootstrap.zip")),
            memorySize: 128,
            role: lambdaRole,
            timeout: cdk.Duration.seconds(2),
            environment: {
                RUST_LOG: 'info,api=debug',
                TOKENS_TABLE: props.token_db_stack.tokensTable.tableName,
                TOKENS_TABLE_TOKENS_INDEX: props.token_db_stack.tokensIndexName,
            },
        });
    }
}
