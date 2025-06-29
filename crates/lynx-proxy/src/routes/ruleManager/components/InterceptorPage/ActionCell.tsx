/* eslint-disable no-case-declarations */
import React from 'react';
import { Tag, Typography } from 'antd';
import {
  HandlerRule,
  HandlerRuleType,
} from '@/services/generated/utoipaAxum.schemas';
import { get } from 'lodash';
import { useI18n } from '@/contexts';

const { Text } = Typography;

interface ActionCellProps {
  handlers: HandlerRule[];
}

// Handler类型对应的标签颜色和显示文本
const useHandlerRuleTypeConfig = (type?: HandlerRuleType) => {
  const { t } = useI18n();
  const config = React.useMemo(() => {
    switch (type?.type) {
      case 'block':
        return { color: 'red', text: t('ruleManager.handlerTypes.block') };
      case 'delay':
        return { color: 'gold', text: t('ruleManager.handlerTypes.delay') };
      case 'modifyRequest':
        return {
          color: 'blue',
          text: t('ruleManager.handlerTypes.modifyRequest'),
        };
      case 'modifyResponse':
        return {
          color: 'green',
          text: t('ruleManager.handlerTypes.modifyResponse'),
        };
      case 'localFile':
        return {
          color: 'orange',
          text: t('ruleManager.handlerTypes.localFile'),
        };
      case 'proxyForward':
        return {
          color: 'purple',
          text: t('ruleManager.handlerTypes.proxyForward'),
        };
      case 'htmlScriptInjector':
        return {
          color: 'cyan',
          text: t('ruleManager.handlerTypes.htmlScriptInjector'),
        };

      default:
        return {
          color: 'default',
          text: t('ruleManager.handlerTypes.unknown'),
        };
    }
  }, [type?.type, t]);

  return config;
};

// 安全地获取config中的值
const getConfigValue = (
  config: HandlerRule,
  key: string,
  defaultValue?: unknown,
) => {
  if (config && typeof config === 'object' && key in config) {
    return get(config, key, defaultValue);
  }
  return defaultValue;
};

// 获取handler的描述文本
const useGetHandlerDescription = (handler: HandlerRule): string => {
  const { t } = useI18n();
  const { handlerType } = handler;

  switch (handlerType.type) {
    case 'block':
      const statusCode = getConfigValue(handler, 'statusCode', 403);
      const reason = getConfigValue(
        handler,
        'reason',
        t('ruleManager.handlerDescriptions.reason'),
      );
      return (
        t('ruleManager.handlerDescriptions.statusCode', { code: statusCode }) +
        ' - ' +
        reason
      );

    case 'delay':
      const delayMs = handlerType.delayMs;
      const delayType = handlerType.delayType;
      const varianceMs = handlerType.varianceMs;

      // 获取延迟类型的国际化文案
      const delayTypeText = t(`ruleManager.createRuleDrawer.handlerBehavior.delayHandler.delayType.${delayType}`);

      let delayText = t('ruleManager.handlerDescriptions.delay', {
        delayMs,
        delayType: delayTypeText
      });

      if (varianceMs) {
        delayText += ` (±${varianceMs}ms)`;
      }

      return delayText;

    case 'modifyRequest':
      const parts = [];
      if (getConfigValue(handler, 'modifyHeaders')) {
        parts.push(t('ruleManager.handlerDescriptions.modifyHeaders'));
      }
      if (getConfigValue(handler, 'modifyBody')) {
        parts.push(t('ruleManager.handlerDescriptions.modifyBody'));
      }
      const modifyMethod = getConfigValue(handler, 'modifyMethod');
      if (modifyMethod) {
        parts.push(
          t('ruleManager.handlerDescriptions.modifyMethod', {
            method: modifyMethod,
          }),
        );
      }
      if (getConfigValue(handler, 'modifyUrl')) {
        parts.push(t('ruleManager.handlerDescriptions.modifyUrl'));
      }
      return parts.length > 0
        ? parts.join(', ')
        : t('ruleManager.handlerDescriptions.modifyRequest');

    case 'modifyResponse':
      const responseParts = [];
      const responseStatusCode = getConfigValue(handler, 'statusCode');
      if (responseStatusCode) {
        responseParts.push(
          t('ruleManager.handlerDescriptions.statusCode', {
            code: responseStatusCode,
          }),
        );
      }
      if (getConfigValue(handler, 'headers')) {
        responseParts.push(
          t('ruleManager.handlerDescriptions.modifyResponseHeaders'),
        );
      }
      if (getConfigValue(handler, 'body')) {
        responseParts.push(
          t('ruleManager.handlerDescriptions.modifyResponseBody'),
        );
      }
      return responseParts.length > 0
        ? responseParts.join(', ')
        : t('ruleManager.handlerDescriptions.modifyResponse');

    case 'localFile':
      const filePath = getConfigValue(handler, 'filePath');
      return filePath
        ? t('ruleManager.handlerDescriptions.file', { path: filePath })
        : t('ruleManager.handlerDescriptions.returnLocalFile');

    case 'proxyForward':
      const targetHost = getConfigValue(handler, 'targetHost');
      return targetHost
        ? t('ruleManager.handlerDescriptions.forwardTo', { host: targetHost })
        : t('ruleManager.handlerDescriptions.proxyForward');

    case 'htmlScriptInjector':
      return t('ruleManager.handlerDescriptions.htmlScriptInjector');

    default:
      return handler.description || t('ruleManager.handlerTypes.unknown');
  }
};

export const ActionCell: React.FC<ActionCellProps> = ({ handlers }) => {
  const { t } = useI18n();

  if (!handlers || handlers.length === 0) {
    return (
      <div>
        <Text type="secondary" style={{ fontSize: '12px' }}>
          {t('ruleManager.noActions')}
        </Text>
      </div>
    );
  }

  // 只显示启用的handlers
  const enabledHandlers = handlers.filter((handler) => handler.enabled);

  if (enabledHandlers.length === 0) {
    return (
      <div>
        <Text type="secondary" style={{ fontSize: '12px' }}>
          {t('ruleManager.allActionsDisabled')}
        </Text>
      </div>
    );
  }

  // 按执行顺序排序
  const sortedHandlers = [...enabledHandlers].sort(
    (a, b) => a.executionOrder - b.executionOrder,
  );

  return (
    <div>
      {sortedHandlers.map((handler, index) => {
        return (
          <ActionItem
            key={handler.id || index}
            handler={handler}
            index={index}
            length={sortedHandlers.length}
          />
        );
      })}
    </div>
  );
};

const ActionItem: React.FC<{
  handler: HandlerRule;
  index: number;
  length: number;
}> = ({ handler, index, length }) => {
  const config = useHandlerRuleTypeConfig(handler?.handlerType);
  const content = useGetHandlerDescription(handler);

  return (
    <div
      key={handler.id || index}
      style={{ marginBottom: index < length - 1 ? 4 : 0 }}
    >
      <Tag color={config.color}>{config.text}</Tag>
      <br />
      <Text type="secondary" style={{ fontSize: '12px' }}>
        {content}
      </Text>
    </div>
  );
};
