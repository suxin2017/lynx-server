import React, { useState } from 'react';
import {
  Card,
  Form,
  Input,
  Select,
  Row,
  Col,
  Button,
  Space,
  Typography,
  Divider,
  Collapse,
  Tooltip,
} from 'antd';
import {
  PlusOutlined,
  DeleteOutlined,
  InfoCircleOutlined,
  BranchesOutlined,
  CopyOutlined,
  ArrowUpOutlined,
  ArrowDownOutlined,
} from '@ant-design/icons';
import {
  CaptureType,
  LogicalOperator,
} from '../../../services/generated/utoipaAxum.schemas';

const { Text, Paragraph } = Typography;
const { Option } = Select;
const { Panel } = Collapse;

export interface ComplexRuleMatchingSectionProps {
  disabled?: boolean;
}

// 条件类型定义
interface SimpleCondition {
  type: 'simple';
  captureType?: CaptureType;
  pattern?: string;
  method?: string;
  host?: string;
}

interface ComplexCondition {
  type: 'complex';
  operator?: LogicalOperator;
  conditions?: (SimpleCondition | ComplexCondition)[];
}

type Condition = SimpleCondition | ComplexCondition;

const logicalOperatorOptions = [
  {
    value: LogicalOperator.and,
    label: 'AND（与）',
    description: '所有条件都必须满足',
    color: '#52c41a',
  },
  {
    value: LogicalOperator.or,
    label: 'OR（或）',
    description: '任一条件满足即可',
    color: '#1890ff',
  },
  {
    value: LogicalOperator.not,
    label: 'NOT（非）',
    description: '条件不满足时匹配',
    color: '#ff4d4f',
  },
];

const captureTypeOptions = [
  { value: CaptureType.glob, label: 'Glob 模式' },
  { value: CaptureType.regex, label: '正则表达式' },
  { value: CaptureType.exact, label: '精确匹配' },
  { value: CaptureType.contains, label: '包含匹配' },
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

interface ConditionFormProps {
  path: (string | number)[];
  onRemove?: () => void;
  onCopy?: () => void;
  onMoveUp?: () => void;
  onMoveDown?: () => void;
  canRemove?: boolean;
  canMoveUp?: boolean;
  canMoveDown?: boolean;
  disabled?: boolean;
  isNested?: boolean;
  index?: number;
  totalCount?: number;
}

const ConditionForm: React.FC<ConditionFormProps> = ({
  path,
  onRemove,
  onCopy,
  onMoveUp,
  onMoveDown,
  canRemove = true,
  canMoveUp = false,
  canMoveDown = false,
  disabled = false,
  isNested = false,
  index = 0,
  totalCount = 1,
}) => {
  const form = Form.useFormInstance();
  const conditionType = Form.useWatch([...path, 'type'], form);

  return (
    <Card
      size="small"
      style={{
        marginBottom: 8,
        backgroundColor: isNested ? '#fafafa' : '#fff',
        border: isNested ? '1px dashed #d9d9d9' : '1px solid #d9d9d9',
      }}
      title={
        <Space>
          <BranchesOutlined />
          <Text>条件配置 #{index + 1}</Text>
          {conditionType && (
            <Text type="secondary">
              ({conditionType === 'simple' ? '简单条件' : '复杂条件'})
            </Text>
          )}
        </Space>
      }
      extra={
        <Space size={4}>
          {totalCount > 1 && canMoveUp && (
            <Tooltip title="上移条件">
              <Button
                type="text"
                icon={<ArrowUpOutlined />}
                onClick={onMoveUp}
                disabled={disabled}
              />
            </Tooltip>
          )}
          {totalCount > 1 && canMoveDown && (
            <Tooltip title="下移条件">
              <Button
                type="text"
                icon={<ArrowDownOutlined />}
                onClick={onMoveDown}
                disabled={disabled}
              />
            </Tooltip>
          )}
          {onCopy && (
            <Tooltip title="复制条件">
              <Button
                type="text"
                icon={<CopyOutlined />}
                onClick={onCopy}
                disabled={disabled}
              />
            </Tooltip>
          )}
          {canRemove && onRemove && (
            <Tooltip title="删除条件">
              <Button
                type="text"
                icon={<DeleteOutlined />}
                onClick={onRemove}
                disabled={disabled}
                danger
              />
            </Tooltip>
          )}
        </Space>
      }
    >
      <Row gutter={[16, 8]}>
        <Col span={24}>
          <Form.Item
            label="条件类型"
            name={[...path, 'type']}
            rules={[{ required: true, message: '请选择条件类型' }]}
          >
            <Select disabled={disabled} placeholder="选择条件类型">
              <Option value="simple">简单条件</Option>
              <Option value="complex">复杂条件</Option>
            </Select>
          </Form.Item>
        </Col>
      </Row>

      {conditionType === 'simple' && (
        <SimpleConditionFields path={path} disabled={disabled} />
      )}

      {conditionType === 'complex' && (
        <ComplexConditionFields path={path} disabled={disabled} />
      )}
    </Card>
  );
};

const SimpleConditionFields: React.FC<{
  path: (string | number)[];
  disabled?: boolean;
}> = ({ path, disabled = false }) => {
  return (
    <>
      <Row gutter={[16, 8]}>
        <Col span={8}>
          <Form.Item
            label="匹配类型"
            name={[...path, 'captureType']}
            rules={[{ required: true, message: '请选择匹配类型' }]}
            initialValue={CaptureType.glob}
          >
            <Select disabled={disabled}>
              {captureTypeOptions.map((option) => (
                <Option key={option.value} value={option.value}>
                  {option.label}
                </Option>
              ))}
            </Select>
          </Form.Item>
        </Col>
        <Col span={16}>
          <Form.Item
            label="匹配模式"
            name={[...path, 'pattern']}
            rules={[{ required: true, message: '请输入匹配模式' }]}
          >
            <Input
              disabled={disabled}
              placeholder="URL 匹配模式，如：/api/*, *.html, /user/{id}"
            />
          </Form.Item>
        </Col>
      </Row>
      <Row gutter={[16, 8]}>
        <Col span={12}>
          <Form.Item label="HTTP 方法" name={[...path, 'method']}>
            <Select
              disabled={disabled}
              placeholder="选择方法（可选）"
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
          <Form.Item label="主机" name={[...path, 'host']}>
            <Input
              disabled={disabled}
              placeholder="主机名（可选），如：api.example.com"
            />
          </Form.Item>
        </Col>
      </Row>
    </>
  );
};

const ComplexConditionFields: React.FC<{
  path: (string | number)[];
  disabled?: boolean;
}> = ({ path, disabled = false }) => {
  const form = Form.useFormInstance();
  const operator = Form.useWatch([...path, 'operator'], form);
  const isNotOperator = operator === LogicalOperator.not;

  return (
    <>
      <Row gutter={[16, 8]}>
        <Col span={24}>
          <Form.Item
            label="逻辑操作符"
            name={[...path, 'operator']}
            rules={[{ required: true, message: '请选择逻辑操作符' }]}
            initialValue={LogicalOperator.and}
          >
            <Select disabled={disabled}>
              {logicalOperatorOptions.map((option) => (
                <Option key={option.value} value={option.value}>
                  <Space>
                    <span style={{ color: option.color }}>●</span>
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

      <Form.List name={[...path, 'conditions']}>
        {(fields, { add, remove, move }) => {
          // 为 NOT 操作符限制子条件数量
          const canAddMore = !isNotOperator || fields.length === 0;

          const copyCondition = (index: number) => {
            const conditions =
              form.getFieldValue([...path, 'conditions']) || [];
            const conditionToCopy = conditions[index];
            if (conditionToCopy) {
              // 深拷贝条件对象
              const copiedCondition = JSON.parse(
                JSON.stringify(conditionToCopy),
              );
              add(copiedCondition);
            }
          };

          const moveCondition = (fromIndex: number, toIndex: number) => {
            move(fromIndex, toIndex);
          };

          return (
            <>
              <Row gutter={[16, 8]}>
                <Col span={24}>
                  <Space direction="vertical" style={{ width: '100%' }}>
                    <Space align="center" wrap>
                      <Text strong>子条件列表</Text>
                      {isNotOperator && (
                        <Text type="warning" style={{ fontSize: '12px' }}>
                          ⚠️ NOT 操作符只能有一个子条件
                        </Text>
                      )}
                      {canAddMore && (
                        <Space>
                          <Tooltip
                            title={`添加简单条件${isNotOperator ? '（NOT 操作符最多一个）' : ''}`}
                          >
                            <Button
                              type="dashed"
                              onClick={() => add({ type: 'simple' })}
                              icon={<PlusOutlined />}
                              disabled={disabled}
                              size="small"
                            >
                              添加简单条件
                            </Button>
                          </Tooltip>
                          <Tooltip
                            title={`添加复杂条件${isNotOperator ? '（NOT 操作符最多一个）' : ''}`}
                          >
                            <Button
                              type="dashed"
                              onClick={() =>
                                add({
                                  type: 'complex',
                                  operator: LogicalOperator.and,
                                  conditions: [],
                                })
                              }
                              icon={<BranchesOutlined />}
                              disabled={disabled}
                              size="small"
                            >
                              添加复杂条件
                            </Button>
                          </Tooltip>
                        </Space>
                      )}
                    </Space>

                    {fields.map((field, index) => (
                      <ConditionForm
                        key={field.key}
                        path={[...path, 'conditions', field.name]}
                        index={index}
                        totalCount={fields.length}
                        onRemove={() => remove(field.name)}
                        onCopy={() => canAddMore && copyCondition(index)}
                        onMoveUp={
                          index > 0 && !isNotOperator
                            ? () => moveCondition(index, index - 1)
                            : undefined
                        }
                        onMoveDown={
                          index < fields.length - 1 && !isNotOperator
                            ? () => moveCondition(index, index + 1)
                            : undefined
                        }
                        canRemove={fields.length > 1 || isNotOperator}
                        canMoveUp={index > 0 && !isNotOperator}
                        canMoveDown={
                          index < fields.length - 1 && !isNotOperator
                        }
                        disabled={disabled}
                        isNested={true}
                      />
                    ))}

                    {fields.length === 0 && (
                      <Card
                        size="small"
                        style={{
                          backgroundColor: '#f5f5f5',
                          border: '1px dashed #d9d9d9',
                        }}
                      >
                        <Text
                          type="secondary"
                          style={{
                            display: 'block',
                            textAlign: 'center',
                            padding: '12px',
                          }}
                        >
                          📝 请至少添加一个子条件来完成逻辑组合
                        </Text>
                      </Card>
                    )}
                  </Space>
                </Col>
              </Row>
            </>
          );
        }}
      </Form.List>
    </>
  );
};

export const ComplexRuleMatchingSection: React.FC<
  ComplexRuleMatchingSectionProps
> = ({ disabled = false }) => {
  const [activeKey, setActiveKey] = useState<string | string[]>(['1']);

  return (
    <Card title="🔧 复杂规则匹配条件">
      <Space direction="vertical" style={{ width: '100%' }} size="middle">
        <Paragraph type="secondary">
          <InfoCircleOutlined />{' '}
          复杂规则支持多个条件的逻辑组合，可以创建嵌套的匹配逻辑。支持条件的排序、复制和移动操作。
        </Paragraph>

        <Collapse activeKey={activeKey} onChange={setActiveKey}>
          <Panel header="逻辑操作符说明" key="1">
            <Space direction="vertical" style={{ width: '100%' }}>
              {logicalOperatorOptions.map((option) => (
                <Row key={option.value} gutter={16} align="middle">
                  <Col span={4}>
                    <Space>
                      <span style={{ color: option.color, fontSize: '14px' }}>
                        ●
                      </span>
                      <Text strong style={{ color: option.color }}>
                        {option.label}
                      </Text>
                    </Space>
                  </Col>
                  <Col span={20}>
                    <Text type="secondary">{option.description}</Text>
                  </Col>
                </Row>
              ))}

              <Divider style={{ margin: '12px 0' }} />

              <Row gutter={16}>
                <Col span={24}>
                  <Text strong>操作说明：</Text>
                  <ul style={{ marginTop: '8px', marginBottom: '0' }}>
                    <li>
                      <Text type="secondary">使用 ↑↓ 按钮调整条件顺序</Text>
                    </li>
                    <li>
                      <Text type="secondary">使用 📋 按钮复制已有条件</Text>
                    </li>
                    <li>
                      <Text type="secondary">NOT 操作符只能包含一个子条件</Text>
                    </li>
                    <li>
                      <Text type="secondary">支持无限层级的嵌套组合</Text>
                    </li>
                  </ul>
                </Col>
              </Row>
            </Space>
          </Panel>
        </Collapse>

        <Form.Item
          name={['capture', 'condition']}
          rules={[{ required: true, message: '请配置匹配条件' }]}
          initialValue={{
            type: 'complex',
            operator: LogicalOperator.and,
            conditions: [],
          }}
        >
          <div style={{ display: 'none' }} />
        </Form.Item>

        <ConditionForm
          path={['capture', 'condition']}
          canRemove={false}
          disabled={disabled}
          index={0}
          totalCount={1}
        />

        <Divider />

        <Space direction="vertical" style={{ width: '100%' }}>
          <Text strong>💡 使用提示：</Text>
          <ul style={{ margin: 0, paddingLeft: '20px' }}>
            <li>
              <Text type="secondary" style={{ fontSize: '12px' }}>
                简单条件：直接匹配 URL、方法、主机等
              </Text>
            </li>
            <li>
              <Text type="secondary" style={{ fontSize: '12px' }}>
                复杂条件：组合多个条件，支持 AND、OR、NOT 逻辑
              </Text>
            </li>
            <li>
              <Text type="secondary" style={{ fontSize: '12px' }}>
                嵌套组合：可在复杂条件中添加更多复杂条件
              </Text>
            </li>
            <li>
              <Text type="secondary" style={{ fontSize: '12px' }}>
                条件编辑：支持排序、复制、删除等操作
              </Text>
            </li>
          </ul>
        </Space>
      </Space>
    </Card>
  );
};

export default ComplexRuleMatchingSection;
