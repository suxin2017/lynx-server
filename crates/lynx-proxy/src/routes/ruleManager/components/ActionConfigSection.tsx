import React from 'react';
import { Card, Form, Input, Select, InputNumber, Space, Button } from 'antd';
import { RiAddLine, RiDeleteBinLine } from '@remixicon/react';

const { TextArea } = Input;
const { Option } = Select;

interface ActionConfigSectionProps {
  actionType: string;
  onActionTypeChange: (value: string) => void;
}

export const ActionConfigSection: React.FC<ActionConfigSectionProps> = ({
  actionType,
  onActionTypeChange,
}) => {
  const renderActionConfig = () => {
    switch (actionType) {
      case 'redirect':
        return (
          <Form.Item
            name="redirectTarget"
            label="重定向目标"
            rules={[{ required: true, message: '请输入重定向目标' }]}
          >
            <Input placeholder="例如: api.example.com" />
          </Form.Item>
        );

      case 'modify_request':
        return (
          <Form.Item label="修改请求头">
            <Form.List name="requestHeaders">
              {(fields, { add, remove }) => (
                <div>
                  {fields.map(({ key, name, ...restField }) => (
                    <Space
                      key={key}
                      style={{ display: 'flex', marginBottom: 8 }}
                      align="baseline"
                    >
                      <Form.Item
                        {...restField}
                        name={[name, 'key']}
                        rules={[
                          { required: true, message: '请输入请求头名称' },
                        ]}
                      >
                        <Input placeholder="请求头名称" />
                      </Form.Item>
                      <Form.Item
                        {...restField}
                        name={[name, 'value']}
                        rules={[{ required: true, message: '请输入请求头值' }]}
                      >
                        <Input placeholder="请求头值" />
                      </Form.Item>
                      <Button
                        type="text"
                        icon={<RiDeleteBinLine size={16} />}
                        onClick={() => remove(name)}
                        danger
                      />
                    </Space>
                  ))}
                  <Form.Item>
                    <Button
                      type="dashed"
                      onClick={() => add()}
                      block
                      icon={<RiAddLine size={16} />}
                    >
                      添加请求头
                    </Button>
                  </Form.Item>
                </div>
              )}
            </Form.List>
          </Form.Item>
        );

      case 'modify_response':
        return (
          <>
            <Form.Item
              name="responseStatusCode"
              label="响应状态码"
              rules={[{ required: true, message: '请输入响应状态码' }]}
            >
              <InputNumber
                min={100}
                max={599}
                placeholder="200"
                style={{ width: '100%' }}
              />
            </Form.Item>
            <Form.Item name="responseBody" label="响应内容">
              <TextArea
                rows={4}
                placeholder="输入响应内容（JSON、HTML、文本等）"
              />
            </Form.Item>
          </>
        );

      default:
        return null;
    }
  };

  return (
    <Card title="执行动作" size="small">
      <Form.Item
        name="actionType"
        label="动作类型"
        rules={[{ required: true, message: '请选择动作类型' }]}
      >
        <Select onChange={onActionTypeChange}>
          <Option value="block">阻止请求</Option>
          <Option value="redirect">请求重定向</Option>
          <Option value="modify_request">修改请求</Option>
          <Option value="modify_response">修改响应</Option>
        </Select>
      </Form.Item>

      {renderActionConfig()}
    </Card>
  );
};
