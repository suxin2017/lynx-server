import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import { useSelector } from 'react-redux';
import { IViewMessageEventStoreValue, RootState } from './useSortPoll';
import { FilterEngine } from '@/routes/network/components/FilterTemplate/filterEngine';
import { useFilterTemplate } from '@/routes/network/components/FilterTemplate/context';
import { ExtendedMessageEventStoreValue } from './messageEventCache';

export interface RequestTableState {
  requests: IViewMessageEventStoreValue[];
  filterUri: string;
  filterMimeType: string[];
  pendingRequestIds: Record<string, boolean>;
}
const initialState: RequestTableState = {
  requests: [],
  filterUri: '',
  filterMimeType: [],
  pendingRequestIds: {},
};

const isCompletedReq = (res: IViewMessageEventStoreValue) => {

  // 隧道代理
  if (res.status === 'Completed' && res.tunnel?.status === 'Disconnected') {
    return true;
  }

  // websocket 断开链接 或者报错了
  if (
    res.status === 'Completed' &&
    res.messages &&
    (res.messages?.status === 'Disconnected' ||
      (typeof res.messages.status === 'object' &&
        'Error' in res.messages.status))
  ) {
    return true;
  }

  // 普通请求完全完成
  if (res.status === 'Completed' && !res.tunnel && !res.messages && res.timings.reponseBodyEnd) {
    return true;
  }

  // 请求报错了
  if (typeof res.status === 'object' && res.status?.Error) {
    return true;
  }
  return false;
};

const requestTableSlice = createSlice({
  name: 'requestTable',
  initialState,

  reducers: {
    clearRequestTable: () => initialState,
    insertOrUpdateRequests: (
      state,
      action: PayloadAction<IViewMessageEventStoreValue[]>,
    ) => {
      action.payload.forEach((item) => {
        const existingIndex = state.requests.findIndex(
          (req) => req.traceId === item.traceId,
        );
        if (existingIndex !== -1) {
          state.requests[existingIndex] = item;
        } else {
          state.requests.push(item);
        }
      });
    },
    appendRequest: (
      state,
      action: PayloadAction<IViewMessageEventStoreValue[]>,
    ) => {
      state.requests.push(...action.payload);
      action.payload
        ?.filter((res) => !isCompletedReq(res))
        ?.map((res) => res.traceId)
        .forEach((id) => {
          state.pendingRequestIds[id] = true;
        });
    },
    replaceRequest: (
      state,
      action: PayloadAction<IViewMessageEventStoreValue[]>,
    ) => {
      state.requests.forEach((request, index) => {
        const newRequest = action.payload.find(
          (newRequest) => newRequest.traceId === request.traceId,
        );
        if (newRequest) {
          if (isCompletedReq(newRequest)) {
            delete state.pendingRequestIds[newRequest.traceId];
          }

          state.requests[index] = newRequest;
        }
      });
    },
    filterUri: (state, action: PayloadAction<string>) => {
      state.filterUri = action.payload;
    },
    filterMimeType: (state, action: PayloadAction<string[]>) => {
      state.filterMimeType = action.payload;
    },
    removeOldRequest: (
      state,
      action: PayloadAction<{
        maxLogSize: number;
      }>,
    ) => {
      state.requests = state.requests.slice(-action.payload.maxLogSize);
    },
  },
});

export const useRequestLogCount = () => {
  return useSelector((state: RootState) => state.requestTable.requests.length);
};
export const useFilteredTableData = () => {
  const { state: filterTemplateState } = useFilterTemplate();
  
  return useSelector((state: RootState) => {
    // 首先应用原有的URI和MIME类型过滤
    let filteredData = state.requestTable.requests
      .filter((requestValue) => {
        if (!state.requestTable.filterUri) {
          return true;
        }
        return requestValue.request?.url?.includes(
          state.requestTable.filterUri,
        );
      });
    

    // 如果过滤模板状态可用，则应用过滤引擎
    if (filterTemplateState?.templates) {
      const filterResult = FilterEngine.filter(
        filteredData as ExtendedMessageEventStoreValue[], 
        filterTemplateState.templates
      );
      return filterResult.filtered as IViewMessageEventStoreValue[];
    }
    
    // 如果过滤模板不可用，直接返回基础过滤结果
    return filteredData;
  });
};

export const {
  insertOrUpdateRequests,
  appendRequest,
  removeOldRequest,
  replaceRequest,
  clearRequestTable,
  filterMimeType,
  filterUri,
} = requestTableSlice.actions;

export const requestTableReducer = requestTableSlice.reducer;
