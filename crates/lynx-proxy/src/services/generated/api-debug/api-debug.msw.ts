/**
 * Generated by orval v7.9.0 🍺
 * Do not edit manually.
 * utoipa-axum
 * Utoipa's axum bindings for seamless integration for the two
 * OpenAPI spec version: 0.2.0
 */
import { faker } from '@faker-js/faker';

import { HttpResponse, delay, http } from 'msw';

import { HttpMethod, RequestStatus, ResponseCode } from '../utoipaAxum.schemas';
import type {
  ResponseDataWrapperApiDebugListResponse,
  ResponseDataWrapperApiDebugResponse,
  ResponseDataWrapperApiDebugStats,
  ResponseDataWrapperTupleUnit,
  ResponseDataWrapperU64,
} from '../utoipaAxum.schemas';

export const getListDebugEntriesResponseMock = (
  overrideResponse: Partial<ResponseDataWrapperApiDebugListResponse> = {},
): ResponseDataWrapperApiDebugListResponse => ({
  code: faker.helpers.arrayElement(Object.values(ResponseCode)),
  data: {
    data: Array.from(
      { length: faker.number.int({ min: 1, max: 10 }) },
      (_, i) => i + 1,
    ).map(() => ({
      body: faker.helpers.arrayElement([
        faker.helpers.arrayElement([faker.string.alpha(20), null]),
        undefined,
      ]),
      contentType: faker.helpers.arrayElement([
        faker.helpers.arrayElement([faker.string.alpha(20), null]),
        undefined,
      ]),
      createdAt: faker.number.int({ min: undefined, max: undefined }),
      errorMessage: faker.helpers.arrayElement([
        faker.helpers.arrayElement([faker.string.alpha(20), null]),
        undefined,
      ]),
      headers: faker.helpers.arrayElement([
        faker.helpers.arrayElement([null]),
        undefined,
      ]),
      id: faker.number.int({ min: undefined, max: undefined }),
      method: faker.helpers.arrayElement(Object.values(HttpMethod)),
      name: faker.string.alpha(20),
      responseBody: faker.helpers.arrayElement([
        faker.helpers.arrayElement([faker.string.alpha(20), null]),
        undefined,
      ]),
      responseHeaders: faker.helpers.arrayElement([
        faker.helpers.arrayElement([null]),
        undefined,
      ]),
      responseStatus: faker.helpers.arrayElement([
        faker.helpers.arrayElement([
          faker.number.int({ min: undefined, max: undefined }),
          null,
        ]),
        undefined,
      ]),
      responseTime: faker.helpers.arrayElement([
        faker.helpers.arrayElement([
          faker.number.int({ min: undefined, max: undefined }),
          null,
        ]),
        undefined,
      ]),
      status: faker.helpers.arrayElement(Object.values(RequestStatus)),
      timeout: faker.helpers.arrayElement([
        faker.helpers.arrayElement([
          faker.number.int({ min: undefined, max: undefined }),
          null,
        ]),
        undefined,
      ]),
      updatedAt: faker.number.int({ min: undefined, max: undefined }),
      url: faker.string.alpha(20),
    })),
    page: faker.number.int({ min: 0, max: undefined }),
    perPage: faker.number.int({ min: 0, max: undefined }),
    total: faker.number.int({ min: 0, max: undefined }),
    totalPages: faker.number.int({ min: 0, max: undefined }),
  },
  message: faker.helpers.arrayElement([
    faker.helpers.arrayElement([faker.string.alpha(20), null]),
    undefined,
  ]),
  ...overrideResponse,
});

export const getCreateDebugEntryResponseMock = (
  overrideResponse: Partial<ResponseDataWrapperApiDebugResponse> = {},
): ResponseDataWrapperApiDebugResponse => ({
  code: faker.helpers.arrayElement(Object.values(ResponseCode)),
  data: {
    body: faker.helpers.arrayElement([
      faker.helpers.arrayElement([faker.string.alpha(20), null]),
      undefined,
    ]),
    contentType: faker.helpers.arrayElement([
      faker.helpers.arrayElement([faker.string.alpha(20), null]),
      undefined,
    ]),
    createdAt: faker.number.int({ min: undefined, max: undefined }),
    errorMessage: faker.helpers.arrayElement([
      faker.helpers.arrayElement([faker.string.alpha(20), null]),
      undefined,
    ]),
    headers: faker.helpers.arrayElement([
      faker.helpers.arrayElement([null]),
      undefined,
    ]),
    id: faker.number.int({ min: undefined, max: undefined }),
    method: faker.helpers.arrayElement(Object.values(HttpMethod)),
    name: faker.string.alpha(20),
    responseBody: faker.helpers.arrayElement([
      faker.helpers.arrayElement([faker.string.alpha(20), null]),
      undefined,
    ]),
    responseHeaders: faker.helpers.arrayElement([
      faker.helpers.arrayElement([null]),
      undefined,
    ]),
    responseStatus: faker.helpers.arrayElement([
      faker.helpers.arrayElement([
        faker.number.int({ min: undefined, max: undefined }),
        null,
      ]),
      undefined,
    ]),
    responseTime: faker.helpers.arrayElement([
      faker.helpers.arrayElement([
        faker.number.int({ min: undefined, max: undefined }),
        null,
      ]),
      undefined,
    ]),
    status: faker.helpers.arrayElement(Object.values(RequestStatus)),
    timeout: faker.helpers.arrayElement([
      faker.helpers.arrayElement([
        faker.number.int({ min: undefined, max: undefined }),
        null,
      ]),
      undefined,
    ]),
    updatedAt: faker.number.int({ min: undefined, max: undefined }),
    url: faker.string.alpha(20),
  },
  message: faker.helpers.arrayElement([
    faker.helpers.arrayElement([faker.string.alpha(20), null]),
    undefined,
  ]),
  ...overrideResponse,
});

export const getClearAllDebugEntriesResponseMock = (
  overrideResponse: Partial<ResponseDataWrapperU64> = {},
): ResponseDataWrapperU64 => ({
  code: faker.helpers.arrayElement(Object.values(ResponseCode)),
  data: faker.number.int({ min: 0, max: undefined }),
  message: faker.helpers.arrayElement([
    faker.helpers.arrayElement([faker.string.alpha(20), null]),
    undefined,
  ]),
  ...overrideResponse,
});

export const getGetDebugStatsResponseMock = (
  overrideResponse: Partial<ResponseDataWrapperApiDebugStats> = {},
): ResponseDataWrapperApiDebugStats => ({
  code: faker.helpers.arrayElement(Object.values(ResponseCode)),
  data: {
    failedCount: faker.number.int({ min: 0, max: undefined }),
    pendingCount: faker.number.int({ min: 0, max: undefined }),
    successCount: faker.number.int({ min: 0, max: undefined }),
    total: faker.number.int({ min: 0, max: undefined }),
  },
  message: faker.helpers.arrayElement([
    faker.helpers.arrayElement([faker.string.alpha(20), null]),
    undefined,
  ]),
  ...overrideResponse,
});

export const getGetDebugEntryResponseMock = (
  overrideResponse: Partial<ResponseDataWrapperApiDebugResponse> = {},
): ResponseDataWrapperApiDebugResponse => ({
  code: faker.helpers.arrayElement(Object.values(ResponseCode)),
  data: {
    body: faker.helpers.arrayElement([
      faker.helpers.arrayElement([faker.string.alpha(20), null]),
      undefined,
    ]),
    contentType: faker.helpers.arrayElement([
      faker.helpers.arrayElement([faker.string.alpha(20), null]),
      undefined,
    ]),
    createdAt: faker.number.int({ min: undefined, max: undefined }),
    errorMessage: faker.helpers.arrayElement([
      faker.helpers.arrayElement([faker.string.alpha(20), null]),
      undefined,
    ]),
    headers: faker.helpers.arrayElement([
      faker.helpers.arrayElement([null]),
      undefined,
    ]),
    id: faker.number.int({ min: undefined, max: undefined }),
    method: faker.helpers.arrayElement(Object.values(HttpMethod)),
    name: faker.string.alpha(20),
    responseBody: faker.helpers.arrayElement([
      faker.helpers.arrayElement([faker.string.alpha(20), null]),
      undefined,
    ]),
    responseHeaders: faker.helpers.arrayElement([
      faker.helpers.arrayElement([null]),
      undefined,
    ]),
    responseStatus: faker.helpers.arrayElement([
      faker.helpers.arrayElement([
        faker.number.int({ min: undefined, max: undefined }),
        null,
      ]),
      undefined,
    ]),
    responseTime: faker.helpers.arrayElement([
      faker.helpers.arrayElement([
        faker.number.int({ min: undefined, max: undefined }),
        null,
      ]),
      undefined,
    ]),
    status: faker.helpers.arrayElement(Object.values(RequestStatus)),
    timeout: faker.helpers.arrayElement([
      faker.helpers.arrayElement([
        faker.number.int({ min: undefined, max: undefined }),
        null,
      ]),
      undefined,
    ]),
    updatedAt: faker.number.int({ min: undefined, max: undefined }),
    url: faker.string.alpha(20),
  },
  message: faker.helpers.arrayElement([
    faker.helpers.arrayElement([faker.string.alpha(20), null]),
    undefined,
  ]),
  ...overrideResponse,
});

export const getUpdateDebugEntryResponseMock = (
  overrideResponse: Partial<ResponseDataWrapperApiDebugResponse> = {},
): ResponseDataWrapperApiDebugResponse => ({
  code: faker.helpers.arrayElement(Object.values(ResponseCode)),
  data: {
    body: faker.helpers.arrayElement([
      faker.helpers.arrayElement([faker.string.alpha(20), null]),
      undefined,
    ]),
    contentType: faker.helpers.arrayElement([
      faker.helpers.arrayElement([faker.string.alpha(20), null]),
      undefined,
    ]),
    createdAt: faker.number.int({ min: undefined, max: undefined }),
    errorMessage: faker.helpers.arrayElement([
      faker.helpers.arrayElement([faker.string.alpha(20), null]),
      undefined,
    ]),
    headers: faker.helpers.arrayElement([
      faker.helpers.arrayElement([null]),
      undefined,
    ]),
    id: faker.number.int({ min: undefined, max: undefined }),
    method: faker.helpers.arrayElement(Object.values(HttpMethod)),
    name: faker.string.alpha(20),
    responseBody: faker.helpers.arrayElement([
      faker.helpers.arrayElement([faker.string.alpha(20), null]),
      undefined,
    ]),
    responseHeaders: faker.helpers.arrayElement([
      faker.helpers.arrayElement([null]),
      undefined,
    ]),
    responseStatus: faker.helpers.arrayElement([
      faker.helpers.arrayElement([
        faker.number.int({ min: undefined, max: undefined }),
        null,
      ]),
      undefined,
    ]),
    responseTime: faker.helpers.arrayElement([
      faker.helpers.arrayElement([
        faker.number.int({ min: undefined, max: undefined }),
        null,
      ]),
      undefined,
    ]),
    status: faker.helpers.arrayElement(Object.values(RequestStatus)),
    timeout: faker.helpers.arrayElement([
      faker.helpers.arrayElement([
        faker.number.int({ min: undefined, max: undefined }),
        null,
      ]),
      undefined,
    ]),
    updatedAt: faker.number.int({ min: undefined, max: undefined }),
    url: faker.string.alpha(20),
  },
  message: faker.helpers.arrayElement([
    faker.helpers.arrayElement([faker.string.alpha(20), null]),
    undefined,
  ]),
  ...overrideResponse,
});

export const getDeleteDebugEntryResponseMock = (
  overrideResponse: Partial<ResponseDataWrapperTupleUnit> = {},
): ResponseDataWrapperTupleUnit => ({
  code: faker.helpers.arrayElement(Object.values(ResponseCode)),
  data: {},
  message: faker.helpers.arrayElement([
    faker.helpers.arrayElement([faker.string.alpha(20), null]),
    undefined,
  ]),
  ...overrideResponse,
});

export const getListDebugEntriesMockHandler = (
  overrideResponse?:
    | ResponseDataWrapperApiDebugListResponse
    | ((
        info: Parameters<Parameters<typeof http.get>[1]>[0],
      ) =>
        | Promise<ResponseDataWrapperApiDebugListResponse>
        | ResponseDataWrapperApiDebugListResponse),
) => {
  return http.get('*/api_debug/debug', async (info) => {
    await delay(1000);

    return new HttpResponse(
      JSON.stringify(
        overrideResponse !== undefined
          ? typeof overrideResponse === 'function'
            ? await overrideResponse(info)
            : overrideResponse
          : getListDebugEntriesResponseMock(),
      ),
      { status: 200, headers: { 'Content-Type': 'application/json' } },
    );
  });
};

export const getCreateDebugEntryMockHandler = (
  overrideResponse?:
    | ResponseDataWrapperApiDebugResponse
    | ((
        info: Parameters<Parameters<typeof http.post>[1]>[0],
      ) =>
        | Promise<ResponseDataWrapperApiDebugResponse>
        | ResponseDataWrapperApiDebugResponse),
) => {
  return http.post('*/api_debug/debug', async (info) => {
    await delay(1000);

    return new HttpResponse(
      JSON.stringify(
        overrideResponse !== undefined
          ? typeof overrideResponse === 'function'
            ? await overrideResponse(info)
            : overrideResponse
          : getCreateDebugEntryResponseMock(),
      ),
      { status: 200, headers: { 'Content-Type': 'application/json' } },
    );
  });
};

export const getClearAllDebugEntriesMockHandler = (
  overrideResponse?:
    | ResponseDataWrapperU64
    | ((
        info: Parameters<Parameters<typeof http.delete>[1]>[0],
      ) => Promise<ResponseDataWrapperU64> | ResponseDataWrapperU64),
) => {
  return http.delete('*/api_debug/debug', async (info) => {
    await delay(1000);

    return new HttpResponse(
      JSON.stringify(
        overrideResponse !== undefined
          ? typeof overrideResponse === 'function'
            ? await overrideResponse(info)
            : overrideResponse
          : getClearAllDebugEntriesResponseMock(),
      ),
      { status: 200, headers: { 'Content-Type': 'application/json' } },
    );
  });
};

export const getGetDebugStatsMockHandler = (
  overrideResponse?:
    | ResponseDataWrapperApiDebugStats
    | ((
        info: Parameters<Parameters<typeof http.get>[1]>[0],
      ) =>
        | Promise<ResponseDataWrapperApiDebugStats>
        | ResponseDataWrapperApiDebugStats),
) => {
  return http.get('*/api_debug/debug/stats', async (info) => {
    await delay(1000);

    return new HttpResponse(
      JSON.stringify(
        overrideResponse !== undefined
          ? typeof overrideResponse === 'function'
            ? await overrideResponse(info)
            : overrideResponse
          : getGetDebugStatsResponseMock(),
      ),
      { status: 200, headers: { 'Content-Type': 'application/json' } },
    );
  });
};

export const getGetDebugEntryMockHandler = (
  overrideResponse?:
    | ResponseDataWrapperApiDebugResponse
    | ((
        info: Parameters<Parameters<typeof http.get>[1]>[0],
      ) =>
        | Promise<ResponseDataWrapperApiDebugResponse>
        | ResponseDataWrapperApiDebugResponse),
) => {
  return http.get('*/api_debug/debug/:id', async (info) => {
    await delay(1000);

    return new HttpResponse(
      JSON.stringify(
        overrideResponse !== undefined
          ? typeof overrideResponse === 'function'
            ? await overrideResponse(info)
            : overrideResponse
          : getGetDebugEntryResponseMock(),
      ),
      { status: 200, headers: { 'Content-Type': 'application/json' } },
    );
  });
};

export const getUpdateDebugEntryMockHandler = (
  overrideResponse?:
    | ResponseDataWrapperApiDebugResponse
    | ((
        info: Parameters<Parameters<typeof http.put>[1]>[0],
      ) =>
        | Promise<ResponseDataWrapperApiDebugResponse>
        | ResponseDataWrapperApiDebugResponse),
) => {
  return http.put('*/api_debug/debug/:id', async (info) => {
    await delay(1000);

    return new HttpResponse(
      JSON.stringify(
        overrideResponse !== undefined
          ? typeof overrideResponse === 'function'
            ? await overrideResponse(info)
            : overrideResponse
          : getUpdateDebugEntryResponseMock(),
      ),
      { status: 200, headers: { 'Content-Type': 'application/json' } },
    );
  });
};

export const getDeleteDebugEntryMockHandler = (
  overrideResponse?:
    | ResponseDataWrapperTupleUnit
    | ((
        info: Parameters<Parameters<typeof http.delete>[1]>[0],
      ) =>
        | Promise<ResponseDataWrapperTupleUnit>
        | ResponseDataWrapperTupleUnit),
) => {
  return http.delete('*/api_debug/debug/:id', async (info) => {
    await delay(1000);

    return new HttpResponse(
      JSON.stringify(
        overrideResponse !== undefined
          ? typeof overrideResponse === 'function'
            ? await overrideResponse(info)
            : overrideResponse
          : getDeleteDebugEntryResponseMock(),
      ),
      { status: 200, headers: { 'Content-Type': 'application/json' } },
    );
  });
};
export const getApiDebugMock = () => [
  getListDebugEntriesMockHandler(),
  getCreateDebugEntryMockHandler(),
  getClearAllDebugEntriesMockHandler(),
  getGetDebugStatsMockHandler(),
  getGetDebugEntryMockHandler(),
  getUpdateDebugEntryMockHandler(),
  getDeleteDebugEntryMockHandler(),
];
