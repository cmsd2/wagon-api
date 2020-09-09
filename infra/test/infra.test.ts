import {
  expect as expectCDK,
  matchTemplate,
  MatchStyle,
  haveResource,
} from "@aws-cdk/assert";
import "@aws-cdk/assert/jest";
import * as cdk from "@aws-cdk/core";
import * as Infra from "../lib/wagon-api-stack";

test("Empty Stack", () => {
  const app = new cdk.App();
  // WHEN
  const stack = new Infra.WagonApiStack(app, "MyTestStack");
  // THEN
  expectCDK(stack).to(haveResource("AWS::IAM::Role", {}));
  expectCDK(stack).to(haveResource("AWS::Lambda::Function", {}));
});
