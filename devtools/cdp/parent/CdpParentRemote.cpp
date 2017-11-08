/* -*- Mode: C++; tab-width: 8; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* vim: set ts=8 sts=2 et sw=2 tw=99: */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include "CdpParentRemote.h"

#include "mozilla/ClearOnShutdown.h"
#include "mozilla/Logging.h"
#include "mozilla/Services.h"

#include "nsXULAppAPI.h"

#include "IdType.h"

using mozilla::dom::ContentParentId;
using mozilla::dom::TabId;

extern "C" {
extern void* gecko_cdp_parent_remote_start(const char*);
extern void gecko_cdp_parent_remote_stop(void*);

extern void gecko_cdp_parent_remote_page_tools_actor_recv(void*, const nsCString*);
extern void gecko_cdp_parent_remote_page_tools_actor_closed(void*);
}

namespace mozilla {
namespace devtools {
namespace cdp {

LazyLogModule gLog("cdp");

StaticAutoPtr<RemoteServer> RemoteServer::sInstance;

nsresult
RemoteServer::Start(const char* portStr)
{
  MOZ_ASSERT(XRE_IsParentProcess());

  if (!sInstance) {
    sInstance = new RemoteServer();
    if (!sInstance)
      return NS_ERROR_OUT_OF_MEMORY;

    ClearOnShutdown(&sInstance, ShutdownPhase::Shutdown);
  }

  // Skip the rest if already started.
  if (sInstance->mServerPtr)
    return NS_OK;

  MOZ_LOG(gLog, LogLevel::Debug, ("Starting CDP remote server..."));
  sInstance->mServerPtr = gecko_cdp_parent_remote_start(portStr);
  if (!sInstance->mServerPtr)
    return NS_ERROR_FAILURE;

  return NS_OK;
}

RemoteServer::RemoteServer()
  : mServerPtr(nullptr)
{
}

RemoteServer::~RemoteServer()
{
  if (mServerPtr) {
    MOZ_LOG(gLog, LogLevel::Debug, ("Stopping CDP remote server..."));
    gecko_cdp_parent_remote_stop(mServerPtr);
  }
  mServerPtr = nullptr;
}

} // namespace cdp
} // namespace devtools
} // namespace mozilla
