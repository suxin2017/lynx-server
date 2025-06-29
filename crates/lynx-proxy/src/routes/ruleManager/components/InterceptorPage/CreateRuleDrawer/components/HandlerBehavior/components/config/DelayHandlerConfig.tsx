import React from 'react';
import { Form, InputNumber, Select, Typography } from 'antd';
import { useI18n } from '@/contexts';

const { Text } = Typography;

interface DelayHandlerConfigProps {
  field: {
    key: number;
    name: number;
  };
}

export const DelayHandlerConfig: React.FC<DelayHandlerConfigProps> = ({
  field,
}) => {
  const { t } = useI18n();

  const delayTypeOptions = [
    {
      value: 'beforeRequest',
      label: t('ruleManager.createRuleDrawer.handlerBehavior.delayHandler.delayType.beforeRequest'),
    },
    {
      value: 'afterRequest',
      label: t('ruleManager.createRuleDrawer.handlerBehavior.delayHandler.delayType.afterRequest'),
    },
    {
      value: 'both',
      label: t('ruleManager.createRuleDrawer.handlerBehavior.delayHandler.delayType.both'),
    },
  ];

  return (
    <div className="space-y-4">
      <Text strong>
        {t('ruleManager.createRuleDrawer.handlerBehavior.delayHandler.title')}
      </Text>

      <div className="grid grid-cols-2 gap-4">
        <Form.Item
          name={[field.name, 'handlerType', 'delayMs']}
          label={t(
            'ruleManager.createRuleDrawer.handlerBehavior.delayHandler.delayMs',
          )}
          rules={[
            {
              required: true,
              message: t(
                'ruleManager.createRuleDrawer.handlerBehavior.delayHandler.delayMsRequired',
              ),
            },
            {
              type: 'number',
              min: 0,
              message: t(
                'ruleManager.createRuleDrawer.handlerBehavior.delayHandler.delayMsMin',
              ),
            },
          ]}
          initialValue={1000}
        >
          <InputNumber
            placeholder="1000"
            min={0}
            max={300000}
            className="w-full"
            addonAfter="ms"
          />
        </Form.Item>

        <Form.Item
          name={[field.name, 'handlerType', 'varianceMs']}
          label={t(
            'ruleManager.createRuleDrawer.handlerBehavior.delayHandler.varianceMs',
          )}
          extra={t(
            'ruleManager.createRuleDrawer.handlerBehavior.delayHandler.varianceMsExtra',
          )}
        >
          <InputNumber
            placeholder="0"
            min={0}
            max={30000}
            className="w-full"
            addonAfter="ms"
          />
        </Form.Item>
      </div>

      <Form.Item
        name={[field.name, 'handlerType', 'delayType']}
        label={t(
          'ruleManager.createRuleDrawer.handlerBehavior.delayHandler.delayTypeLabel',
        )}
        initialValue="beforeRequest"
      >
        <Select
          placeholder={t(
            'ruleManager.createRuleDrawer.handlerBehavior.delayHandler.delayTypePlaceholder',
          )}
          options={delayTypeOptions}
        />
      </Form.Item>

      <div className="text-sm text-gray-500 space-y-1">
        <div>• {t('ruleManager.createRuleDrawer.handlerBehavior.delayHandler.tips.tip1')}</div>
        <div>• {t('ruleManager.createRuleDrawer.handlerBehavior.delayHandler.tips.tip2')}</div>
        <div>• {t('ruleManager.createRuleDrawer.handlerBehavior.delayHandler.tips.tip3')}</div>
      </div>

      <div className="text-sm text-gray-500">
        <Text type="secondary">
          {t(
            'ruleManager.createRuleDrawer.handlerBehavior.delayHandler.description',
          )}
        </Text>
      </div>
    </div>
  );
};
