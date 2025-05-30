import { CommonCard } from '@/routes/settings/components/CommonCard';
import {
  RiAddLine,
  RiDeleteBinLine,
  RiEditLine,
  RiFileCopyLine,
  RiMoreLine,
} from '@remixicon/react';
import {
  Button,
  Dropdown,
  Modal,
  Space,
  Switch,
  Table,
  Tag,
  Typography,
  message,
} from 'antd';
import type { ColumnsType } from 'antd/es/table';
import React, { useState } from 'react';
import { CreateRuleDrawer } from './CreateRuleDrawer';

const { Title, Text } = Typography;

export interface InterceptRule {
  id: string;
  name: string;
  description: string;
  matchConditions: {
    url?: string;
    method?: string;
    headers?: Record<string, string>;
  };
  action: {
    type: 'block' | 'redirect' | 'modify_request' | 'modify_response';
    config: Record<string, unknown>;
  };
  priority: number;
  enabled: boolean;
  createdAt: string;
  updatedAt: string;
}

// 模拟数据
const mockRules: InterceptRule[] = [
  {
    id: '1',
    name: '阻止广告请求',
    description: '阻止常见的广告域名请求',
    matchConditions: {
      url: 'ads\\.example\\.com|analytics\\.tracker\\.com',
    },
    action: {
      type: 'block',
      config: {},
    },
    priority: 100,
    enabled: true,
    createdAt: '2025-05-04T23:47:22.000Z',
    updatedAt: '2025-05-04T23:47:22.000Z',
  },
  {
    id: '2',
    name: '修改用户代理',
    description: '将所有请求的User-Agent修改为移动设备',
    matchConditions: {
      url: '.*',
    },
    action: {
      type: 'modify_request',
      config: {
        headers: {
          'User-Agent':
            'Mozilla/5.0 (iPhone; CPU iPhone OS 14_7_1 like Mac OS X)',
        },
      },
    },
    priority: 90,
    enabled: true,
    createdAt: '2025-05-04T23:47:22.000Z',
    updatedAt: '2025-05-04T23:47:22.000Z',
  },
  {
    id: '3',
    name: '替换请求体中的文件路径',
    description: '替换请求体中的文件路径并使用键值存储',
    matchConditions: {
      url: 'api\\.example\\.com/upload',
      method: 'POST',
    },
    action: {
      type: 'modify_request',
      config: {
        body: 'replace_file_path',
      },
    },
    priority: 85,
    enabled: true,
    createdAt: '2025-05-04T23:47:22.000Z',
    updatedAt: '2025-05-04T23:47:22.000Z',
  },
  {
    id: '4',
    name: '模拟API响应',
    description: '返回模拟的API响应数据',
    matchConditions: {
      url: 'api\\.example\\.com/users',
      method: 'GET',
    },
    action: {
      type: 'modify_response',
      config: {
        statusCode: 200,
        body: '{"users": [{"id": 1, "name": "Mock User"}]}',
      },
    },
    priority: 80,
    enabled: true,
    createdAt: '2025-05-04T23:47:22.000Z',
    updatedAt: '2025-05-04T23:47:22.000Z',
  },
  {
    id: '5',
    name: '请求重定向',
    description: '将测试环境请求重定向到生产环境',
    matchConditions: {
      url: 'test-api\\.example\\.com',
    },
    action: {
      type: 'redirect',
      config: {
        target: 'api.example.com',
      },
    },
    priority: 70,
    enabled: true,
    createdAt: '2025-05-04T23:47:22.000Z',
    updatedAt: '2025-05-04T23:47:22.000Z',
  },
];

export const InterceptorPage: React.FC = () => {
  const [rules, setRules] = useState<InterceptRule[]>(mockRules);
  const [createModalVisible, setCreateModalVisible] = useState(false);
  const [editModalVisible, setEditModalVisible] = useState(false);
  const [selectedRule, setSelectedRule] = useState<InterceptRule | null>(null);

  const handleCreateRule = (
    rule: Omit<InterceptRule, 'id' | 'createdAt' | 'updatedAt'>,
  ) => {
    const newRule: InterceptRule = {
      ...rule,
      id: Date.now().toString(),
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };
    setRules([...rules, newRule]);
    message.success('规则创建成功');
  };

  const handleEditRule = (rule: InterceptRule) => {
    setRules(
      rules.map((r) =>
        r.id === rule.id ? { ...rule, updatedAt: new Date().toISOString() } : r,
      ),
    );
    message.success('规则更新成功');
  };

  const handleDeleteRule = (id: string) => {
    Modal.confirm({
      title: '确认删除',
      content: '确定要删除这个规则吗？',
      onOk: () => {
        setRules(rules.filter((r) => r.id !== id));
        message.success('规则删除成功');
      },
    });
  };

  const handleToggleRule = (id: string) => {
    setRules(
      rules.map((r) =>
        r.id === id
          ? { ...r, enabled: !r.enabled, updatedAt: new Date().toISOString() }
          : r,
      ),
    );
  };

  const handleCopyRule = (rule: InterceptRule) => {
    const newRule: InterceptRule = {
      ...rule,
      id: Date.now().toString(),
      name: `${rule.name} (副本)`,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };
    setRules([...rules, newRule]);
    message.success('规则复制成功');
  };

  const handleExportRules = () => {
    const dataStr = JSON.stringify(rules, null, 2);
    const dataUri =
      'data:application/json;charset=utf-8,' + encodeURIComponent(dataStr);

    const exportFileDefaultName = `interceptor-rules-${new Date().toISOString().split('T')[0]}.json`;

    const linkElement = document.createElement('a');
    linkElement.setAttribute('href', dataUri);
    linkElement.setAttribute('download', exportFileDefaultName);
    linkElement.click();

    message.success('规则导出成功');
  };

  const handleImportRules = () => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.json';
    input.onchange = (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (file) {
        const reader = new FileReader();
        reader.onload = (event) => {
          try {
            const importedRules = JSON.parse(event.target?.result as string);
            setRules([...rules, ...importedRules]);
            message.success('规则导入成功');
          } catch (_error) {
            message.error('导入失败：文件格式错误');
          }
        };
        reader.readAsText(file);
      }
    };
    input.click();
  };

  const getActionTypeTag = (type: string) => {
    const typeMap = {
      block: { color: 'red', text: '阻止请求' },
      redirect: { color: 'blue', text: '请求重定向' },
      modify_request: { color: 'orange', text: '修改请求' },
      modify_response: { color: 'green', text: '修改响应' },
    };
    const config = typeMap[type as keyof typeof typeMap] || {
      color: 'default',
      text: type,
    };
    return <Tag color={config.color}>{config.text}</Tag>;
  };

  const getConditionsText = (conditions: InterceptRule['matchConditions']) => {
    const parts = [];
    if (conditions.url) parts.push(`URL:${conditions.url}`);
    if (conditions.method) parts.push(`方法:${conditions.method}`);
    return parts.join(' ');
  };

  const getActionDescription = (action: InterceptRule['action']) => {
    switch (action.type) {
      case 'block':
        return '阻止请求';
      case 'redirect':
        return `重定向至: ${action.config.target}`;
      case 'modify_request':
        if (action.config.headers) {
          const headerCount = Object.keys(action.config.headers).length;
          return `修改 ${headerCount} 个请求头`;
        }
        return '修改请求';
      case 'modify_response':
        return `状态码: ${action.config.statusCode}`;
      default:
        return action.type;
    }
  };

  const columns: ColumnsType<InterceptRule> = [
    {
      title: '状态',
      dataIndex: 'enabled',
      key: 'enabled',
      width: 80,
      render: (enabled: boolean, record) => (
        <Switch
        // size="small"
        // onClick={() => handleToggleRule(record.id)}
        // style={{ color: enabled ? '#52c41a' : '#d9d9d9' }}
        />
      ),
    },
    {
      title: '规则名称',
      dataIndex: 'name',
      key: 'name',
      width: 200,
      render: (name: string, record) => (
        <div>
          <Text strong>{name}</Text>
          <br />
          <Text type="secondary" style={{ fontSize: '12px' }}>
            {record.description}
          </Text>
        </div>
      ),
    },
    {
      title: '匹配条件',
      key: 'conditions',
      width: 300,
      render: (_, record) => (
        <Text code style={{ fontSize: '12px' }}>
          {getConditionsText(record.matchConditions)}
        </Text>
      ),
    },
    {
      title: '动作',
      key: 'action',
      width: 200,
      render: (_, record) => (
        <div>
          {getActionTypeTag(record.action.type)}
          <br />
          <Text type="secondary" style={{ fontSize: '12px' }}>
            {getActionDescription(record.action)}
          </Text>
        </div>
      ),
    },
    {
      title: '优先级',
      dataIndex: 'priority',
      key: 'priority',
      width: 80,
      sorter: (a, b) => a.priority - b.priority,
    },
    {
      title: '更新时间',
      dataIndex: 'updatedAt',
      key: 'updatedAt',
      width: 150,
      render: (date: string) => new Date(date).toLocaleString(),
    },
    {
      title: '操作',
      key: 'actions',
      width: 120,
      render: (_, record) => {
        const items = [
          {
            key: 'edit',
            icon: <RiEditLine size={14} />,
            label: '编辑',
            onClick: () => {
              setSelectedRule(record);
              setEditModalVisible(true);
            },
          },
          {
            key: 'copy',
            icon: <RiFileCopyLine size={14} />,
            label: '复制',
            onClick: () => handleCopyRule(record),
          },
          {
            key: 'delete',
            icon: <RiDeleteBinLine size={14} />,
            label: '删除',
            danger: true,
            onClick: () => handleDeleteRule(record.id),
          },
        ];

        return (
          <Dropdown menu={{ items }} trigger={['click']}>
            <Button type="text" icon={<RiMoreLine size={16} />} />
          </Dropdown>
        );
      },
    },
  ];

  return (
    <>
      <CommonCard>
        <div className="mb-4 flex items-center justify-between">
          <Title level={4} style={{ margin: 0 }}>
            拦截规则列表
          </Title>
          <Space>
            <Button
              type="primary"
              icon={<RiAddLine size={16} />}
              onClick={() => setCreateModalVisible(true)}
            >
              新建规则
            </Button>
          </Space>
        </div>

        <Text type="secondary" className="mb-4 block">
          管理用于拦截和修改网络请求的规则
        </Text>

        <Table
          columns={columns}
          dataSource={rules}
          rowKey="id"
          pagination={{
            showSizeChanger: true,
            showQuickJumper: true,
            showTotal: (total) => `共 ${total} 条规则`,
          }}
          size="middle"
        />
      </CommonCard>

      <CreateRuleDrawer
        visible={createModalVisible}
        onCancel={() => setCreateModalVisible(false)}
        onOk={handleCreateRule}
      />
    </>
  );
};
