import {
  expect as expectCDK,
  matchTemplate,
  MatchStyle,
  haveResource,
} from "@aws-cdk/assert";
import "@aws-cdk/assert/jest";
import * as cdk from "aws-cdk-lib";
import { DashboardStack } from "../lib/dashboard-stack";
import { IndexerStack } from "../lib/indexer-stack";
import * as Infra from "../lib/wagon-api-stack";

test("Empty Stack", () => {
  const app = new cdk.App();
  // WHEN
  const dashboardStack = new DashboardStack(app, "MyTestDashboard", {name: "test"});
  const stack = new Infra.WagonApiStack(app, "MyTestStack", {dashboard: dashboardStack.dashboard});
  const indexerStack = new IndexerStack(app, "MyTestIndexer", {});
  // THEN
  expectCDK(stack).to(haveResource("AWS::IAM::Role", {}));
  expectCDK(stack).to(haveResource("AWS::Lambda::Function", {}));
});
