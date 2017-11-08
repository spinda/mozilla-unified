/* -*- Mode: C++; tab-width: 8; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* vim: set ts=8 sts=2 et sw=2 tw=99: */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include "CdpRemotePageToolsParent.h"

#include "mozilla/RefPtr.h"

#include "ContentProcessManager.h"
#include "IdType.h"
#include "TabParent.h"

using namespace mozilla;
using namespace mozilla::dom;
using namespace mozilla::devtools::cdp;
using namespace mozilla::ipc;

using mozilla::Unused;

extern "C" {
void gecko_cdp_parent_remote_page_tools_actor_recv(void* actorSenderPtr, const nsCString* msg);
void gecko_cdp_parent_remote_page_tools_actor_closed(void* actorSenderPtr);
}

namespace mozilla {
namespace devtools {
namespace cdp {

CdpRemotePageToolsParent::CdpRemotePageToolsParent()
  : mActorPtr(nullptr)
  , mActorSenderPtr(nullptr)
{
}

void
CdpRemotePageToolsParent::Init(CdpRemotePageToolsParent** actorPtr, void* actorSenderPtr)
{
  mActorPtr = actorPtr;
  mActorSenderPtr = actorSenderPtr;
}

void
CdpRemotePageToolsParent::ActorDestroy(ActorDestroyReason why)
{
  if (mActorPtr) {
    *mActorPtr = nullptr;
    mActorPtr = nullptr;
  }

  if (mActorSenderPtr) {
    gecko_cdp_parent_remote_page_tools_actor_closed(mActorSenderPtr);
    mActorSenderPtr = nullptr;
  }
}

IPCResult
CdpRemotePageToolsParent::RecvOutgoing(const nsCString& msg)
{
  if (mActorSenderPtr) {
    gecko_cdp_parent_remote_page_tools_actor_recv(mActorSenderPtr, &msg);
  }
  return IPC_OK();
}

} // namespace cdp
} // namespace devtools
} // namespace mozilla

extern "C" void
gecko_cdp_parent_remote_page_tools_actor_start(ContentParentId contentParentId,
                                               TabId tabId,
                                               void* actorSenderPtr,
                                               CdpRemotePageToolsParent*** actorPtr)
{
  ContentProcessManager* cpm = ContentProcessManager::GetSingleton();
  if (!cpm) {
    gecko_cdp_parent_remote_page_tools_actor_closed(actorSenderPtr);
    return;
  }

  CdpRemotePageToolsParent** actorHolder = new CdpRemotePageToolsParent*(nullptr);
  if (!actorHolder) {
    gecko_cdp_parent_remote_page_tools_actor_closed(actorSenderPtr);
    return;
  }

  CdpRemotePageToolsParent* actor = nullptr;

  RefPtr<mozilla::dom::TabParent> tabParent =
    cpm->GetTopLevelTabParentByProcessAndTabId(ContentParentId(contentParentId), TabId(tabId));

  if (tabParent) {
    actor = static_cast<CdpRemotePageToolsParent*>(tabParent->SendPCdpRemotePageToolsConstructor());
  }

  if (!actor) {
    delete actorHolder;
    gecko_cdp_parent_remote_page_tools_actor_closed(actorSenderPtr);
    return;
  }

  *actorHolder = actor;
  actor->Init(actorHolder, actorSenderPtr);
  *actorPtr = actorHolder;
}

extern "C" bool
gecko_cdp_parent_remote_page_tools_actor_send(CdpRemotePageToolsParent** actorPtr,
                                              const nsCString* msg)
{
  CdpRemotePageToolsParent* actor = *actorPtr;
  if (actor) {
    return actor->SendIncoming(*msg);
  }
  return false;
}

extern "C" void
gecko_cdp_parent_remote_page_tools_actor_close(CdpRemotePageToolsParent** actorPtr)
{
  CdpRemotePageToolsParent* actor = *actorPtr;
  if (actor) {
    Unused << NS_WARN_IF(!actor->Send__delete__(actor));
  }
  delete actorPtr;
}
