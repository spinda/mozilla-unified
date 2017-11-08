/* -*- Mode: C++; tab-width: 8; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* vim: set ts=8 sts=2 et sw=2 tw=99: */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#ifndef mozilla_devtools_cdp_CdpRemoteServer_h
#define mozilla_devtools_cdp_CdpRemoteServer_h

#include "mozilla/StaticPtr.h"

#include "nsError.h"

namespace mozilla {
namespace devtools {
namespace cdp {

class RemoteServer final {
public:
  // Start up the remote CDP server on a new thread, if not already started,
  // with an optional port specifier string. Call only from the parent process.
  static nsresult Start(const char* portStr);

  ~RemoteServer();

private:
  RemoteServer();

  RemoteServer(const RemoteServer&) = delete;
  void operator=(const RemoteServer&) = delete;

  void* mServerPtr;

  static StaticAutoPtr<RemoteServer> sInstance;
};

} // namespace cdp
} // namespace devtools
} // namespace mozilla

#endif // mozilla_devtools_cdp_CdpRemoteServer_h
