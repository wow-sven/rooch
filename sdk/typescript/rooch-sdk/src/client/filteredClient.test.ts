// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { describe, it, expect, vi, beforeEach } from 'vitest'
import { IClient } from './interface'
import {
  FilteredProvider,
  ITransactionFilter,
  ITransactionFilterChain,
  FuncFilter,
  FilterFunc,
} from './filteredClient'
import { AnnotatedFunctionResultView, StateView, StatePageView } from '../types'
import { ExecuteViewFunctionParams, ListStatesParams } from './roochClientTypes'

const mockFilter: ITransactionFilter = {
  init: vi.fn(),
  doFilter: vi.fn((request, chain) => chain.doFilter(request)),
  destroy: vi.fn(),
}

const mockProvider: IClient = {
  sendRawTransaction: vi.fn(() => Promise.resolve('mockTransactionId')),

  getRpcApiVersion: function (): Promise<string | undefined> {
    throw new Error('Function not implemented.')
  },
  getChainId: function (): number {
    throw new Error('Function not implemented.')
  },
  executeViewFunction: function (
    params: ExecuteViewFunctionParams,
  ): Promise<AnnotatedFunctionResultView> {
    throw new Error('Function not implemented.')
  },
  getStates: function (accessPath: string): Promise<StateView | null[]> {
    console.log(accessPath)
    throw new Error('Function not implemented.')
  },
  listStates: function (params: ListStatesParams): Promise<StatePageView> {
    console.log(params.accessPath, params.cursor as any, params.limit)
    throw new Error('Function not implemented.')
  },
}

const errorHandlingFilter: FilterFunc = async (request: any, chain: ITransactionFilterChain) => {
  try {
    return await chain.doFilter(request)
  } catch (error) {
    return 'errorHandledTransactionId'
  }
}

describe('FilteredProvider', () => {
  let filteredProvider: FilteredProvider

  beforeEach(() => {
    filteredProvider = new FilteredProvider(mockProvider, [mockFilter])
  })

  it('should call filter and provider correctly when sendRawTransaction', async () => {
    const playload = new Uint8Array()
    const result = await filteredProvider.sendRawTransaction(playload)

    expect(mockFilter.doFilter).toHaveBeenCalledWith(playload, expect.anything())
    expect(mockProvider.sendRawTransaction).toHaveBeenCalledWith(playload)
    expect(result).toBe('mockTransactionId')
  })

  it('should handle error correctly when sendRawTransaction throws error', async () => {
    mockProvider.sendRawTransaction = vi.fn(() => Promise.reject(new Error('mock error')))

    const errorHandlingProvider = new FilteredProvider(mockProvider, [
      new FuncFilter(errorHandlingFilter),
    ])

    const playload = new Uint8Array()
    const result = await errorHandlingProvider.sendRawTransaction(playload)

    expect(mockProvider.sendRawTransaction).toHaveBeenCalledWith(playload)
    expect(result).toBe('errorHandledTransactionId')
  })
})
