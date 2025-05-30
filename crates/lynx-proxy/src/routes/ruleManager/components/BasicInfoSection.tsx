import React from 'react';
import { Card, Form, Input, InputNumber, Switch, Row, Col } from 'antd';

interface BasicInfoSectionProps {
  // 不需要额外的 props，组件直接使用 Form.useFormInstance() 来访问表单
}

export const BasicInfoSection: React.FC<BasicInfoSectionProps> = () => {
  return (
    <Card title="基本信息" size="small" style={{ marginBottom: 16 }}>
      <Row gutter={16}>
        <Col span={12}>
          <Form.Item
            name="name"
            label="规则名称"
            rules={[{ required: true, message: '请输入规则名称' }]}
          >
            <Input placeholder="输入规则名称" />
          </Form.Item>
        </Col>
        <Col span={12}>
          <Form.Item
            name="priority"
            label="优先级"
            rules={[{ required: true, message: '请输入优先级' }]}
          >
            <InputNumber
              min={1}
              max={100}
              placeholder="1-100，数字越大优先级越高"
              style={{ width: '100%' }}
            />
          </Form.Item>
        </Col>
      </Row>

      <Form.Item name="description" label="规则描述">
        <Input placeholder="输入规则描述" />
      </Form.Item>

      <Form.Item name="enabled" label="启用状态" valuePropName="checked">
        <Switch />
      </Form.Item>
    </Card>
  );
};
