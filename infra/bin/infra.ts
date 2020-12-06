#!/usr/bin/env node
import "source-map-support/register";
import * as cdk from "@aws-cdk/core";
import { WagonApiStack } from "../lib/wagon-api-stack";
import { DashboardStack } from "../lib/dashboard-stack";
import { IndexerStack } from "../lib/indexer-stack";
import { env } from "process";
import { TokensDbStack } from "../lib/tokens-db-stack";

const app = new cdk.App();
const dashboard = new DashboardStack(app, "WagonDashboard", {name: "wagon"});
const tokensDb = new TokensDbStack(app, "WagonTokensDb", {
    dashboard: dashboard.dashboard,
});
const apiStack = new WagonApiStack(app, "WagonApi", {
    dashboard: dashboard.dashboard,
    openid_aud: process.env.OPENID_AUD!,
    openid_config_uri: process.env.OPENID_CONFIG_URI!,
    tokens_db_stack: tokensDb,
});
new IndexerStack(app, "WagonIndexer", {});
