import React from 'react';
import {
  Card,
  Form,
  Input,
  Select,
  Row,
  Col,
  Typography,
  Space,
  Divider,
} from 'antd';
import { InfoCircleOutlined } from '@ant-design/icons';
import { CaptureType } from '../../../services/generated/utoipaAxum.schemas';

const { Text, Paragraph } = Typography;
const { Option } = Select;

export interface SimpleRuleMatchingSectionProps {
  disabled?: boolean;
}

const captureTypeOptions = [
  {
    value: CaptureType.glob,
    label: 'Glob 模式',
    description: '支持通配符匹配，如 /api/* 或 **/*.json',
  },
  {
    value: CaptureType.regex,
    label: '正则表达式',
    description: '支持复杂的正则表达式匹配',
  },
  {
    value: CaptureType.exact,
    label: '精确匹配',
    description: '完全匹配指定的字符串',
  },
  {
    value: CaptureType.contains,
    label: '包含匹配',
    description: '检查是否包含指定的子字符串',
  },
];

const httpMethods = [
  'GET',
  'POST',
  'PUT',
  'DELETE',
  'PATCH',
  'HEAD',
  'OPTIONS',
];

export const SimpleRuleMatchingSection: React.FC<
  SimpleRuleMatchingSectionProps
> = ({ disabled = false }) => {
  const form = Form.useFormInstance();

  const selectedCaptureType = Form.useWatch(
    ['capture', 'condition', 'captureType'],
    form,
  );

  const renderPatternHelper = () => {
    switch (selectedCaptureType) {
      case CaptureType.glob:
        return (
          <Space direction="vertical" size="small">
            <Text type="secondary">
              <InfoCircleOutlined /> Glob 语法示例：
            </Text>
            <Text code>/api/*</Text> - 匹配 /api/ 下的所有路径
            <Text code>**/*.json</Text> - 匹配所有 .json 文件
            <Text code>/user/[0-9]*</Text> - 匹配 /user/ 后跟数字的路径
          </Space>
        );
      case CaptureType.regex:
        return (
          <Space direction="vertical" size="small">
            <Text type="secondary">
              <InfoCircleOutlined /> 正则表达式示例：
            </Text>
            <Text code>^/api/v[1-9]/.*$</Text> - 匹配 API 版本路径
            <Text code>.*\.(jpg|png|gif)$</Text> - 匹配图片文件
          </Space>
        );
      case CaptureType.exact:
        return (
          <Text type="secondary">
            <InfoCircleOutlined /> 精确匹配：输入的字符串必须完全一致
          </Text>
        );
      case CaptureType.contains:
        return (
          <Text type="secondary">
            <InfoCircleOutlined /> 包含匹配：URL 中包含输入的字符串即可匹配
          </Text>
        );
      default:
        return null;
    }
  };

  return (
    <Card title="简单规则匹配条件">
      <Row gutter={16}>
        <Col span={24}>
          <Form.Item
            label="匹配类型"
            name={['capture', 'condition', 'captureType']}
            rules={[{ required: true, message: '请选择匹配类型' }]}
            initialValue={CaptureType.glob}
          >
            <Select disabled={disabled} placeholder="选择匹配类型">
              {captureTypeOptions.map((option) => (
                <Option key={option.value} value={option.value}>
                  <Space direction="vertical" size={0}>
                    <Text>{option.label}</Text>
                    <Text type="secondary" style={{ fontSize: '12px' }}>
                      {option.description}
                    </Text>
                  </Space>
                </Option>
              ))}
            </Select>
          </Form.Item>
        </Col>
      </Row>

      <Row gutter={16}>
        <Col span={24}>
          <Form.Item
            label="URL 匹配模式"
            name={['capture', 'condition', 'pattern']}
            rules={[
              { required: true, message: '请输入 URL 匹配模式' },
              { min: 1, message: '匹配模式不能为空' },
            ]}
          >
            <Input.TextArea
              disabled={disabled}
              placeholder="输入 URL 匹配模式"
              rows={2}
              showCount
              maxLength={500}
            />
          </Form.Item>
          {renderPatternHelper()}
        </Col>
      </Row>

      <Divider orientation="left" orientationMargin="0">
        <Text type="secondary">可选过滤条件</Text>
      </Divider>

      <Row gutter={16}>
        <Col span={12}>
          <Form.Item
            label="HTTP 方法"
            name={['capture', 'condition', 'method']}
            tooltip="留空表示匹配所有 HTTP 方法"
          >
            <Select
              disabled={disabled}
              placeholder="选择 HTTP 方法（可选）"
              allowClear
            >
              {httpMethods.map((method) => (
                <Option key={method} value={method}>
                  {method}
                </Option>
              ))}
            </Select>
          </Form.Item>
        </Col>
        <Col span={12}>
          <Form.Item
            label="主机过滤"
            name={['capture', 'condition', 'host']}
            tooltip="过滤特定主机的请求，留空表示匹配所有主机"
          >
            <Input disabled={disabled} placeholder="example.com（可选）" />
          </Form.Item>
        </Col>
      </Row>

      <Row gutter={16}>
        <Col span={24}>
          <Paragraph type="secondary" style={{ fontSize: '12px', margin: 0 }}>
            <InfoCircleOutlined />{' '}
            简单规则适用于大多数基础匹配需求。如需更复杂的逻辑组合，请使用复杂规则。
          </Paragraph>
        </Col>
      </Row>
    </Card>
  );
};

export default SimpleRuleMatchingSection;
