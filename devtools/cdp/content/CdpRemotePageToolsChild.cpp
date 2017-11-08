/* -*- Mode: C++; tab-width: 8; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* vim: set ts=8 sts=2 et sw=2 tw=99: */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include "CdpRemotePageToolsChild.h"

using namespace mozilla;
using namespace mozilla::devtools::cdp;
using namespace mozilla::ipc;

extern "C" {
void* gecko_cdp_content_remote_page_tools_actor_opened(CdpRemotePageToolsChild** actorPtr);
void gecko_cdp_content_remote_page_tools_actor_recv(void* toolsPtr, const nsCString* msg);
void gecko_cdp_content_remote_page_tools_actor_closed(void* toolsPtr);
}

namespace mozilla {
namespace devtools {
namespace cdp {

/* static */ PCdpRemotePageToolsChild*
CdpRemotePageToolsChild::Create()
{
  CdpRemotePageToolsChild** actorPtr = new CdpRemotePageToolsChild*(nullptr);
  if (!actorPtr) {
    return nullptr;
  }

  CdpRemotePageToolsChild* actor = new CdpRemotePageToolsChild(actorPtr);
  if (!actor) {
    delete actorPtr;
  }
  return actor;
}

CdpRemotePageToolsChild::CdpRemotePageToolsChild(CdpRemotePageToolsChild** actorPtr)
{
  *actorPtr = this;
  mActorPtr = actorPtr;
  mToolsPtr = gecko_cdp_content_remote_page_tools_actor_opened(actorPtr);
}

void
CdpRemotePageToolsChild::ActorDestroy(ActorDestroyReason why)
{
  if (mActorPtr) {
    *mActorPtr = nullptr;
    mActorPtr = nullptr;
  }

  if (mToolsPtr) {
    gecko_cdp_content_remote_page_tools_actor_closed(mToolsPtr);
    mToolsPtr = nullptr;
  }
}

IPCResult
CdpRemotePageToolsChild::RecvIncoming(const nsCString& msg)
{
  if (mToolsPtr) {
    gecko_cdp_content_remote_page_tools_actor_recv(mToolsPtr, &msg);
  }
  return IPC_OK();
}

void
CdpRemotePageToolsChild::Drop()
{
  mActorPtr = nullptr;
}

} // namespace cdp
} // namespace devtools
} // namespace mozilla

extern "C" bool
gecko_cdp_content_remote_page_tools_actor_send(CdpRemotePageToolsChild** actorPtr,
                                               const nsCString* msg)
{
  CdpRemotePageToolsChild* actor = *actorPtr;
  if (actor) {
    return actor->SendOutgoing(*msg);
  }
  return false;
}

extern "C" void
gecko_cdp_content_remote_page_tools_actor_drop(CdpRemotePageToolsChild** actorPtr)
{
  CdpRemotePageToolsChild* actor = *actorPtr;
  if (actor) {
    actor->Drop();
  }
  delete actorPtr;
}
