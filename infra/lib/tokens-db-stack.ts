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

export interface TokensApiStackProps extends cdk.StackProps {
  dashboard: cw.Dashboard,
}

export class TokensDbStack extends cdk.Stack {
    tokensTable: ddb.Table;
    tokensIndexName: string;

    constructor(scope: Construct, id: string, props: TokensApiStackProps) {
        super(scope, id, props);

        this.tokensTable = new ddb.Table(this, 'Tokens', {
            partitionKey: {
                name: 'user_id', type: ddb.AttributeType.STRING
            },
            billingMode: ddb.BillingMode.PAY_PER_REQUEST,
            encryption: ddb.TableEncryption.DEFAULT,
        });

        this.tokensIndexName = 'TokensIndex';
        this.tokensTable.addGlobalSecondaryIndex({
            indexName: this.tokensIndexName,
            partitionKey: {
                name: 'token',
                type: ddb.AttributeType.STRING,
            },
            projectionType: ddb.ProjectionType.KEYS_ONLY,
        });
    }
}
