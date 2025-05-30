import { Card, Radio, Space, Typography } from 'antd';

const { Text } = Typography;

export interface RuleTypeSelectorProps {
  ruleType: 'simple' | 'complex';
  onRuleTypeChange: (type: 'simple' | 'complex') => void;
  disabled?: boolean;
}

export const RuleTypeSelector: React.FC<RuleTypeSelectorProps> = ({
  ruleType,
  onRuleTypeChange,
  disabled = false,
}) => {
  return (
    <Card size="small" title="规则类型选择">
      <Space direction="vertical" size="middle">
        <Radio.Group
          value={ruleType}
          onChange={(e) => onRuleTypeChange(e.target.value)}
          disabled={disabled}
        >
          <Space direction="vertical">
            <Radio value="simple">
              <Space direction="vertical" size={0}>
                <Text strong>简单规则</Text>
                <Text type="secondary" style={{ fontSize: '12px' }}>
                  单个匹配条件，支持 URL 模式、HTTP 方法和主机过滤
                </Text>
              </Space>
            </Radio>
            <Radio value="complex">
              <Space direction="vertical" size={0}>
                <Text strong>复杂规则</Text>
                <Text type="secondary" style={{ fontSize: '12px' }}>
                  支持逻辑操作符（AND、OR、NOT）的多条件组合规则
                </Text>
              </Space>
            </Radio>
          </Space>
        </Radio.Group>
      </Space>
    </Card>
  );
};

export default RuleTypeSelector;
