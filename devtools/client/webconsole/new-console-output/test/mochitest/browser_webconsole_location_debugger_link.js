/* -*- indent-tabs-mode: nil; js-indent-level: 2 -*- */
/* vim: set ft=javascript ts=2 et sw=2 tw=80: */
/* Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/publicdomain/zero/1.0/ */

// Test that message source links for js errors and console API calls open in
// the jsdebugger when clicked.

"use strict";
requestLongerTimeout(2);

const TEST_URI = "http://example.com/browser/devtools/client/webconsole/" +
  "new-console-output/test/mochitest/test-location-debugger-link.html";

add_task(function* () {
  // Force the new debugger UI, in case this gets uplifted with the old
  // debugger still turned on
  yield pushPref("devtools.debugger.new-debugger-frontend", true);
  yield pushPref("devtools.webconsole.filter.error", true);
  yield pushPref("devtools.webconsole.filter.log", true);

  // On e10s, the exception thrown in test-location-debugger-link-errors.js
  // is triggered in child process and is ignored by test harness
  if (!Services.appinfo.browserTabsRemoteAutostart) {
    expectUncaughtException();
  }

  let hud = yield openNewTabAndConsole(TEST_URI);
  let target = TargetFactory.forTab(gBrowser.selectedTab);
  let toolbox = gDevTools.getToolbox(target);

  yield testOpenInDebugger(hud, toolbox, "document.bar");

  info("Selecting the console again");
  yield toolbox.selectTool("webconsole");
  yield testOpenInDebugger(hud, toolbox, "Blah Blah");

  // // check again the first node.
  info("Selecting the console again");
  yield toolbox.selectTool("webconsole");
  yield testOpenInDebugger(hud, toolbox, "document.bar");
});
