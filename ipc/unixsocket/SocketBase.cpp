/* -*- Mode: c++; c-basic-offset: 2; indent-tabs-mode: nil; tab-width: 40 -*- */
/* vim: set ts=2 et sw=2 tw=80: */

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#include "SocketBase.h"
#include <errno.h>
#include <string.h>
#include <unistd.h>

namespace mozilla {
namespace ipc {

//
// UnixSocketRawData
//

UnixSocketRawData::UnixSocketRawData(size_t aSize)
: mSize(0)
, mCurrentWriteOffset(0)
, mAvailableSpace(aSize)
{
  mData = new uint8_t[mAvailableSpace];
}

UnixSocketRawData::UnixSocketRawData(const void* aData, size_t aSize)
: mSize(aSize)
, mCurrentWriteOffset(0)
, mAvailableSpace(aSize)
{
  MOZ_ASSERT(aData || !mSize);

  mData = new uint8_t[mAvailableSpace];
  memcpy(mData, aData, mSize);
}

nsresult
UnixSocketRawData::Receive(int aFd)
{
  if (!GetTrailingSpace()) {
    if (!GetLeadingSpace()) {
      return NS_ERROR_OUT_OF_MEMORY; /* buffer is full */
    }
    /* free up space at the end of data buffer */
    if (GetSize() <= GetLeadingSpace()) {
      memcpy(mData, GetData(), GetSize());
    } else {
      memmove(mData, GetData(), GetSize());
    }
    mCurrentWriteOffset = 0;
  }

  ssize_t res =
    TEMP_FAILURE_RETRY(read(aFd, GetTrailingBytes(), GetTrailingSpace()));

  if (res < 0) {
    if (errno == EAGAIN || errno == EWOULDBLOCK) {
      return NS_OK; /* no more data available; try again later */
    }
    return NS_ERROR_FAILURE;
  } else if (!res) {
    /* EOF or peer shutdown sending */
    return NS_OK;
  }

  mSize += res;

  return NS_OK;
}

nsresult
UnixSocketRawData::Send(int aFd)
{
  if (!GetSize()) {
    return NS_OK;
  }

  ssize_t res = TEMP_FAILURE_RETRY(write(aFd, GetData(), GetSize()));

  if (res < 0) {
    if (errno == EAGAIN || errno == EWOULDBLOCK) {
      return NS_OK; /* socket is blocked; try again later */
    }
    return NS_ERROR_FAILURE;
  } else if (!res) {
    /* nothing written */
    return NS_OK;
  }

  Consume(res);

  return NS_OK;
}

//
// SocketConsumerBase
//

SocketConsumerBase::~SocketConsumerBase()
{
  MOZ_ASSERT(mConnectionStatus == SOCKET_DISCONNECTED);
}

SocketConnectionStatus
SocketConsumerBase::GetConnectionStatus() const
{
  MOZ_ASSERT(NS_IsMainThread());

  return mConnectionStatus;
}

int
SocketConsumerBase::GetSuggestedConnectDelayMs() const
{
  MOZ_ASSERT(NS_IsMainThread());

  return mConnectDelayMs;
}

void
SocketConsumerBase::NotifySuccess()
{
  MOZ_ASSERT(NS_IsMainThread());

  mConnectionStatus = SOCKET_CONNECTED;
  mConnectTimestamp = PR_IntervalNow();
  OnConnectSuccess();
}

void
SocketConsumerBase::NotifyError()
{
  MOZ_ASSERT(NS_IsMainThread());

  mConnectionStatus = SOCKET_DISCONNECTED;
  mConnectDelayMs = CalculateConnectDelayMs();
  mConnectTimestamp = 0;
  OnConnectError();
}

void
SocketConsumerBase::NotifyDisconnect()
{
  MOZ_ASSERT(NS_IsMainThread());

  mConnectionStatus = SOCKET_DISCONNECTED;
  mConnectDelayMs = CalculateConnectDelayMs();
  mConnectTimestamp = 0;
  OnDisconnect();
}

uint32_t
SocketConsumerBase::CalculateConnectDelayMs() const
{
  MOZ_ASSERT(NS_IsMainThread());

  uint32_t connectDelayMs = mConnectDelayMs;

  if (mConnectTimestamp && (PR_IntervalNow()-mConnectTimestamp) > connectDelayMs) {
    // reset delay if connection has been opened for a while, or...
    connectDelayMs = 0;
  } else if (!connectDelayMs) {
    // ...start with a delay of ~1 sec, or...
    connectDelayMs = 1<<10;
  } else if (connectDelayMs < (1<<16)) {
    // ...otherwise increase delay by a factor of 2
    connectDelayMs <<= 1;
  }
  return connectDelayMs;
}

SocketConsumerBase::SocketConsumerBase()
: mConnectionStatus(SOCKET_DISCONNECTED)
, mConnectTimestamp(0)
, mConnectDelayMs(0)
{ }

void
SocketConsumerBase::SetConnectionStatus(
  SocketConnectionStatus aConnectionStatus)
{
  mConnectionStatus = aConnectionStatus;
}

//
// SocketIOBase
//

SocketIOBase::~SocketIOBase()
{ }

void
SocketIOBase::EnqueueData(UnixSocketRawData* aData)
{
  if (!aData->mSize) {
    delete aData; // delete empty data immediately
    return;
  }
  mOutgoingQ.AppendElement(aData);
}

bool
SocketIOBase::HasPendingData() const
{
  return !mOutgoingQ.IsEmpty();
}

SocketIOBase::SocketIOBase(size_t aMaxReadSize)
  : mMaxReadSize(aMaxReadSize)
{
  MOZ_ASSERT(mMaxReadSize);
}

}
}
