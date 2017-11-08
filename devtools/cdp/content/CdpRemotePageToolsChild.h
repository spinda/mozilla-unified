/* -*- Mode: C++; tab-width: 8; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* vim: set ts=8 sts=2 et sw=2 tw=99: */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#ifndef mozilla_devtools_CdpRemotePageToolsChild_h
#define mozilla_devtools_CdpRemotePageToolsChild_h

#include "mozilla/devtools/cdp/PCdpRemotePageToolsChild.h"

namespace mozilla {
namespace devtools {
namespace cdp {

class CdpRemotePageToolsChild
  : public PCdpRemotePageToolsChild
{
  explicit CdpRemotePageToolsChild(CdpRemotePageToolsChild** actorPtr);
  void ActorDestroy(ActorDestroyReason why) override;

  mozilla::ipc::IPCResult RecvIncoming(const nsCString& msg) override;

  CdpRemotePageToolsChild** mActorPtr;
  void* mToolsPtr;

public:
  static PCdpRemotePageToolsChild* Create();
  void Drop();
};

} // namespace cdp
} // namespace devtools
} // namespace mozilla

#endif // mozilla_devtools_CdpRemotePageToolsChild_h
