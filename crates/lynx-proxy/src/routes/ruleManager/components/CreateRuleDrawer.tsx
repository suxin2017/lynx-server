import React, { useState } from 'react';
import { Drawer, Form, Space, Button } from 'antd';
import type { InterceptRule } from './InterceptorPage';
import { BasicInfoSection } from './BasicInfoSection';
import { RuleTypeSelector } from './RuleTypeSelector';
import { SimpleRuleMatchingSection } from './SimpleRuleMatchingSection';
import { ComplexRuleMatchingSection } from './ComplexRuleMatchingSection';
import { ActionConfigSection } from './ActionConfigSection';
import type { FormValues } from './types';
import { CaptureType } from '../../../services/generated/utoipaAxum.schemas';

interface CreateRuleDrawerProps {
  visible: boolean;
  onCancel: () => void;
  onOk: (rule: Omit<InterceptRule, 'id' | 'createdAt' | 'updatedAt'>) => void;
}

export const CreateRuleDrawer: React.FC<CreateRuleDrawerProps> = ({
  visible,
  onCancel,
  onOk,
}) => {
  const [form] = Form.useForm<FormValues>();
  const [actionType, setActionType] = useState<string>('block');
  const [ruleType, setRuleType] = useState<'simple' | 'complex'>('simple');

  const handleOk = async () => {
    try {
      const values = await form.validateFields();

      // 转换为旧的 InterceptRule 格式以保持兼容性
      const matchConditions: InterceptRule['matchConditions'] = {
        url: values.urlPattern || '',
      };

      if (values.method) {
        matchConditions.method = values.method;
      }

      if (values.headers && values.headers.length > 0) {
        matchConditions.headers = values.headers.reduce(
          (acc, header) => {
            if (header.key && header.value) {
              acc[header.key] = header.value;
            }
            return acc;
          },
          {} as Record<string, string>,
        );
      }

      let actionConfig: Record<string, unknown> = {};

      switch (values.actionType) {
        case 'redirect':
          actionConfig = { target: values.redirectTarget };
          break;
        case 'modify_request':
          if (values.requestHeaders && values.requestHeaders.length > 0) {
            actionConfig.headers = values.requestHeaders.reduce(
              (acc, header) => {
                if (header.key && header.value) {
                  acc[header.key] = header.value;
                }
                return acc;
              },
              {} as Record<string, string>,
            );
          }
          break;
        case 'modify_response':
          actionConfig = {
            statusCode: values.responseStatusCode || 200,
            body: values.responseBody || '',
          };
          break;
      }

      const rule: Omit<InterceptRule, 'id' | 'createdAt' | 'updatedAt'> = {
        name: values.name || '',
        description: values.description || '',
        matchConditions,
        action: {
          type: values.actionType || 'block',
          config: actionConfig,
        },
        priority: values.priority || 50,
        enabled: values.enabled || true,
      };

      onOk(rule);
      form.resetFields();
      setActionType('block');
    } catch (error) {
      console.error('Validation failed:', error);
    }
  };

  const handleCancel = () => {
    form.resetFields();
    setActionType('block');
    setRuleType('simple');
    onCancel();
  };

  return (
    <Drawer
      title="创建拦截规则"
      open={visible}
      onClose={handleCancel}
      width={720}
      destroyOnClose
      extra={
        <Space>
          <Button onClick={handleCancel}>取消</Button>
          <Button type="primary" onClick={handleOk}>
            创建规则
          </Button>
        </Space>
      }
    >
      <Form
        form={form}
        layout="vertical"
        initialValues={{
          actionType: 'block',
          priority: 50,
          enabled: true,
          ruleType: 'simple',
          capture: {
            condition: {
              type: 'simple',
              captureType: CaptureType.glob,
              pattern: '',
              config: {},
            },
          },
          handlers: [],
        }}
      >
        <BasicInfoSection />

        <RuleTypeSelector ruleType={ruleType} onRuleTypeChange={setRuleType} />

        {ruleType === 'simple' ? (
          <SimpleRuleMatchingSection />
        ) : (
          <ComplexRuleMatchingSection />
        )}

        <ActionConfigSection
          actionType={actionType}
          onActionTypeChange={setActionType}
        />
      </Form>
    </Drawer>
  );
};
