/* -*- Mode: C++; tab-width: 8; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* vim: set ts=8 sts=2 et sw=2 tw=99: */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include "nsIHttpProtocolHandler.h"
#include "nsIProtocolHandler.h"
#include "nsIRunnable.h"
#include "nsIXULAppInfo.h"
#include "nsServiceManagerUtils.h"
#include "nsThreadUtils.h"

#include "ContentProcessManager.h"
#include "IdType.h"

using namespace mozilla;
using namespace mozilla::dom;

class CdpRunnable final
  : public Runnable
{
public:
  CdpRunnable(void (*callback)(void*), void* data_ptr)
    : Runnable("CdpRunnable")
    , mCallback(callback)
    , mDataPtr(data_ptr)
  {
  }

  NS_IMETHOD Run() override
  {
    mCallback(mDataPtr);
    return NS_OK;
  }

private:
  void (*mCallback)(void*);
  void* mDataPtr;
};

extern "C" void
gecko_cdp_parent_bridge_dispatch_to_main_thread(void (*callback)(void*), void* data_ptr)
{
  nsCOMPtr<nsIRunnable> runnable = new CdpRunnable(callback, data_ptr);
  NS_DispatchToMainThread(runnable);
}

struct GeckoInfo final {
  nsCString name;
  nsCString version;
  nsCString userAgent;
};

extern "C" nsresult
gecko_cdp_parent_bridge_fetch_gecko_info(GeckoInfo* geckoInfo)
{
  nsresult rv;

  nsCOMPtr<nsIXULAppInfo> appInfo = do_GetService("@mozilla.org/xre/app-info;1", &rv);
  if (NS_FAILED(rv))
    return rv;

  nsCOMPtr<nsIHttpProtocolHandler> http =
    do_GetService(NS_NETWORK_PROTOCOL_CONTRACTID_PREFIX "http", &rv);
  if (NS_FAILED(rv))
    return rv;

  rv = http->GetUserAgent(geckoInfo->userAgent);
  if (NS_FAILED(rv))
    return rv;

  appInfo->GetName(geckoInfo->name);
  appInfo->GetVersion(geckoInfo->version);

  return NS_OK;
}

template<typename T>
struct IdIterator final {
  nsTArray<IdType<T>> ids;
  size_t index;
};

extern "C" bool
gecko_cdp_parent_bridge_id_iterator_next(IdIterator<void>* iterator, IdType<void>* item)
{
  size_t index = iterator->index++;
  if (index < iterator->ids.Length()) {
    *item = iterator->ids[index];
    return true;
  } else {
    return false;
  }
}

extern "C" void
gecko_cdp_parent_bridge_id_iterator_drop(IdIterator<void>* iterator)
{
  delete iterator;
}

extern "C" IdIterator<ContentParent>*
gecko_cdp_parent_bridge_fetch_content_parent_ids()
{
  ContentProcessManager* cpm = ContentProcessManager::GetSingleton();
  if (!cpm) {
    return nullptr;
  }

  IdIterator<ContentParent>* iterator = new IdIterator<ContentParent>();
  if (iterator) {
    iterator->ids = Move(cpm->GetTopLevelContentProcesses());
  }
  return iterator;
}

extern "C" IdIterator<TabParent>*
gecko_cdp_parent_bridge_fetch_tab_ids_from_content_parent(ContentParentId contentParentId)
{
  ContentProcessManager* cpm = ContentProcessManager::GetSingleton();
  if (!cpm) {
    return nullptr;
  }

  IdIterator<TabParent>* iterator = new IdIterator<TabParent>();
  if (iterator) {
    iterator->ids = Move(cpm->GetTabParentsByProcessId(contentParentId));
  }
  return iterator;
}
