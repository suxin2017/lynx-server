import { HandlerRule } from '@/services/generated/utoipaAxum.schemas';
import {
  CheckOutlined,
  CloseOutlined,
  DeleteOutlined,
  EditOutlined,
} from '@ant-design/icons';
import { Button, Card, Form, Switch, Typography } from 'antd';
import React from 'react';
import { HandlerConfig } from './config';
import { useI18n } from '@/contexts';

const { Text } = Typography;

interface HandlerItemProps {
  field: {
    key: number;
    name: number;
  };
  index: number;
  isEditing: boolean;
  onEdit: () => void;
  onSave: () => void;
  onCancel: () => void;
  onDelete: () => void;
  isDragging?: boolean;
}

export const HandlerItem: React.FC<HandlerItemProps> = React.memo(
  ({ field, index: _, isEditing, onEdit, onSave, onCancel, onDelete }) => {
    const form = Form.useFormInstance();
    const handlerData: HandlerRule = Form.useWatch(
      ['handlers', field.name],
      form,
    );
    const { t } = useI18n();

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const getHandlerTypeDisplayName = (handlerType: any) => {
      if (!handlerType?.type) return t('ruleManager.handlerTypes.unknown');

      const typeMap = {
        block: t('ruleManager.handlerTypes.block'),
        delay: t('ruleManager.handlerTypes.delay'),
        modifyRequest: t('ruleManager.handlerTypes.modifyRequest'),
        modifyResponse: t('ruleManager.handlerTypes.modifyResponse'),
        localFile: t('ruleManager.handlerTypes.localFile'),
        proxyForward: t('ruleManager.handlerTypes.proxyForward'),
        htmlScriptInjector: t('ruleManager.handlerTypes.htmlScriptInjector'),
      };

      return (
        typeMap[handlerType.type as keyof typeof typeMap] ||
        t('ruleManager.handlerTypes.unknown')
      );
    };

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const getHandlerDescription = (handlerType: any) => {
      if (!handlerType?.type) return '';

      const descMap = {
        block:
          t('ruleManager.handlerDescriptions.statusCode', {
            code: handlerType.statusCode || 403,
          }) +
          ', ' +
          t('ruleManager.handlerDescriptions.reason'),
        delay: t('ruleManager.handlerDescriptions.delay', {
          delayMs: handlerType.delayMs || 1000,
          delayType: handlerType.delayType || 'beforeRequest',
        }),
        modifyRequest: t('ruleManager.handlerDescriptions.modifyRequest'),
        modifyResponse: t('ruleManager.handlerDescriptions.modifyResponse'),
        localFile: t('ruleManager.handlerDescriptions.file', {
          path:
            handlerType.filePath ||
            t(
              'ruleManager.createRuleDrawer.handlerBehavior.handlerItem.notSet',
            ),
        }),
        proxyForward: t('ruleManager.handlerDescriptions.forwardTo', {
          host:
            handlerType.proxyUrl ||
            t(
              'ruleManager.createRuleDrawer.handlerBehavior.handlerItem.notSet',
            ),
        }),
        htmlScriptInjector: t(
          'ruleManager.handlerDescriptions.htmlScriptInjector',
        ),
      };

      return descMap[handlerType.type as keyof typeof descMap] || '';
    };

    return (
      <Card
        size="small"
        className={`handler-item transition-all duration-200`}
        title={
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-3">
              <div className="flex items-center space-x-2">
                <Text strong>
                  {getHandlerTypeDisplayName(handlerData?.handlerType)}
                </Text>
              </div>
            </div>
            <div className="flex items-center space-x-2">
              <Form.Item
                name={[field.name, 'enabled']}
                valuePropName="checked"
                noStyle
              >
                <Switch size="small" />
              </Form.Item>
              {!isEditing ? (
                <>
                  <Button
                    type="text"
                    size="small"
                    icon={<EditOutlined />}
                    onClick={onEdit}
                  />
                  <Button
                    type="text"
                    size="small"
                    danger
                    icon={<DeleteOutlined />}
                    onClick={onDelete}
                  />
                </>
              ) : (
                <>
                  <Button
                    type="text"
                    size="small"
                    icon={<CheckOutlined />}
                    onClick={onSave}
                  />
                  <Button
                    type="text"
                    size="small"
                    icon={<CloseOutlined />}
                    onClick={onCancel}
                  />
                </>
              )}
            </div>
          </div>
        }
      >
        {!isEditing ? (
          <div className="space-y-2">
            <div>
              <Text strong>
                {t(
                  'ruleManager.createRuleDrawer.handlerBehavior.handlerItem.name',
                )}
                :{' '}
              </Text>
              <Text>
                {handlerData?.name ||
                  t(
                    'ruleManager.createRuleDrawer.handlerBehavior.handlerItem.unnamed',
                  )}
              </Text>
            </div>
            {handlerData?.description && (
              <div>
                <Text strong>
                  {t(
                    'ruleManager.createRuleDrawer.handlerBehavior.handlerItem.description',
                  )}
                  :{' '}
                </Text>
                <Text type="secondary">{handlerData.description}</Text>
              </div>
            )}
            <div>
              <Text strong>
                {t(
                  'ruleManager.createRuleDrawer.handlerBehavior.handlerItem.configuration',
                )}
                :{' '}
              </Text>
              <Text type="secondary">
                {getHandlerDescription(handlerData?.handlerType)}
              </Text>
            </div>
          </div>
        ) : (
          <div className="space-y-4">
            {/* 处理器配置部分 */}
            <HandlerConfig field={field} handler={handlerData.handlerType} />
          </div>
        )}
      </Card>
    );
  },
);

HandlerItem.displayName = 'HandlerItem';
