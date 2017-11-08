/* -*- Mode: C++; tab-width: 8; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* vim: set ts=8 sts=2 et sw=2 tw=99: */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#ifndef mozilla_devtools_CdpRemotePageToolsParent_h
#define mozilla_devtools_CdpRemotePageToolsParent_h

#include "mozilla/devtools/cdp/PCdpRemotePageToolsParent.h"

namespace mozilla {
namespace devtools {
namespace cdp {

class CdpRemotePageToolsParent
  : public PCdpRemotePageToolsParent
{
  explicit CdpRemotePageToolsParent();
  void ActorDestroy(ActorDestroyReason why) override;

  mozilla::ipc::IPCResult RecvOutgoing(const nsCString& msg) override;

  CdpRemotePageToolsParent** mActorPtr;
  void* mActorSenderPtr;

public:
  static inline PCdpRemotePageToolsParent* Create();
  void Init(CdpRemotePageToolsParent** actorPtr, void* actorSenderPtr);
};

/* static */ inline PCdpRemotePageToolsParent*
CdpRemotePageToolsParent::Create()
{
  return new CdpRemotePageToolsParent();
}

} // namespace cdp
} // namespace devtools
} // namespace mozilla

#endif // mozilla_devtools_CdpRemotePageToolsParent_h
