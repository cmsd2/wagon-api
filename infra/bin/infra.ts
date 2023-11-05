#!/usr/bin/env node
import "source-map-support/register";
import * as cdk from "aws-cdk-lib";
import { WagonApiStack } from "../lib/wagon-api-stack";
import { DashboardStack } from "../lib/dashboard-stack";
import { IndexerStack } from "../lib/indexer-stack";
import { env } from "process";
import { TokensDbStack } from "../lib/tokens-db-stack";
import { AuthStack } from "../lib/auth-stack";
import { CertStack } from "../lib/cert-stack";
import { SwaggerStack } from "../lib/swagger-stack";

const certParamName = "/wagon/api/cert";

const app = new cdk.App();
const dashboard = new DashboardStack(app, "WagonDashboard", {name: "wagon"});
const tokensDb = new TokensDbStack(app, "WagonTokensDb", {
    dashboard: dashboard.dashboard,
});
const authStack = new AuthStack(app, "WagonAuth", {
    user_pool_id: env.USER_POOL_ID!,
});
const swaggerStack = new SwaggerStack(app, "WagonSwagger", {
    user_pool_id: env.USER_POOL_ID!,
    resource_server: authStack.resource_server,
    scope: authStack.full_access_scope,
});
const certStack = new CertStack(app, "WagonCert", {
    apiDomain: "api",
    zoneId: "Z01082942PS49FCF86EV4",
    zoneName: "octomonkey.cloud",
    paramName: certParamName,
    env: {
        region: "us-east-1"
    },
});
const apiStack = new WagonApiStack(app, "WagonApi", {
    dashboard: dashboard.dashboard,
    tokens_db_stack: tokensDb,
    user_pool_id: env.USER_POOL_ID!,
    apiDomain: "api",
    zoneName: "octomonkey.cloud",
    certParamName: certParamName,
});
