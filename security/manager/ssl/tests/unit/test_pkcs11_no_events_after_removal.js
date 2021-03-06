/* -*- indent-tabs-mode: nil; js-indent-level: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
"use strict";

// This test loads a testing PKCS #11 module that simulates a token being
// inserted and removed from a slot every 50ms. This causes the observer
// service to broadcast the observation topics "smartcard-insert" and
// "smartcard-remove", respectively. This test ensures that these events
// are no longer emitted once the module has been unloaded.

// Ensure that the appropriate initialization has happened.
do_get_profile();
Cc["@mozilla.org/psm;1"].getService(Ci.nsISupports);

function run_test() {
  let pkcs11ModuleDB = Cc["@mozilla.org/security/pkcs11moduledb;1"]
                         .getService(Ci.nsIPKCS11ModuleDB);
  loadPKCS11TestModule(true);
  pkcs11ModuleDB.deleteModule("PKCS11 Test Module");
  Services.obs.addObserver(function() {
    ok(false, "smartcard-insert event should not have been emitted");
  }, "smartcard-insert");
  Services.obs.addObserver(function() {
    ok(false, "smartcard-remove event should not have been emitted");
  }, "smartcard-remove");
  do_timeout(500, do_test_finished);
  do_test_pending();
}
