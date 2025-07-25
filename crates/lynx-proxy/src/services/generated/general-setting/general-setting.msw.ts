/**
 * Generated by orval v7.9.0 🍺
 * Do not edit manually.
 * utoipa-axum
 * Utoipa's axum bindings for seamless integration for the two
 * OpenAPI spec version: 0.2.0
 */
import { faker } from '@faker-js/faker';

import { HttpResponse, delay, http } from 'msw';

import { ConnectType, ResponseCode } from '../utoipaAxum.schemas';
import type {
  ResponseDataWrapperGeneralSetting,
  ResponseDataWrapperTupleUnit,
} from '../utoipaAxum.schemas';

export const getGetGeneralSettingResponseMock = (
  overrideResponse: Partial<ResponseDataWrapperGeneralSetting> = {},
): ResponseDataWrapperGeneralSetting => ({
  code: faker.helpers.arrayElement(Object.values(ResponseCode)),
  data: {
    connectType: faker.helpers.arrayElement(Object.values(ConnectType)),
    language: faker.string.alpha(20),
    maxLogSize: faker.number.int({ min: undefined, max: undefined }),
  },
  message: faker.helpers.arrayElement([
    faker.helpers.arrayElement([faker.string.alpha(20), null]),
    undefined,
  ]),
  ...overrideResponse,
});

export const getUpdateGeneralSettingResponseMock = (
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

export const getGetGeneralSettingMockHandler = (
  overrideResponse?:
    | ResponseDataWrapperGeneralSetting
    | ((
        info: Parameters<Parameters<typeof http.get>[1]>[0],
      ) =>
        | Promise<ResponseDataWrapperGeneralSetting>
        | ResponseDataWrapperGeneralSetting),
) => {
  return http.get('*/general_setting/general-setting', async (info) => {
    await delay(1000);

    return new HttpResponse(
      JSON.stringify(
        overrideResponse !== undefined
          ? typeof overrideResponse === 'function'
            ? await overrideResponse(info)
            : overrideResponse
          : getGetGeneralSettingResponseMock(),
      ),
      { status: 200, headers: { 'Content-Type': 'application/json' } },
    );
  });
};

export const getUpdateGeneralSettingMockHandler = (
  overrideResponse?:
    | ResponseDataWrapperTupleUnit
    | ((
        info: Parameters<Parameters<typeof http.put>[1]>[0],
      ) =>
        | Promise<ResponseDataWrapperTupleUnit>
        | ResponseDataWrapperTupleUnit),
) => {
  return http.put('*/general_setting/general-setting', async (info) => {
    await delay(1000);

    return new HttpResponse(
      JSON.stringify(
        overrideResponse !== undefined
          ? typeof overrideResponse === 'function'
            ? await overrideResponse(info)
            : overrideResponse
          : getUpdateGeneralSettingResponseMock(),
      ),
      { status: 200, headers: { 'Content-Type': 'application/json' } },
    );
  });
};
export const getGeneralSettingMock = () => [
  getGetGeneralSettingMockHandler(),
  getUpdateGeneralSettingMockHandler(),
];
