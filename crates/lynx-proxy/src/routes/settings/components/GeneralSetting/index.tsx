import { LanguageSelector } from '@/components/LanguageSelector';
import { useGeneralSetting } from '@/store/useGeneralState';
import { Button, Form, InputNumber, message, Space, Typography } from 'antd';
import React from 'react';
import { useI18n } from '@/contexts';
import { CommonCard } from '../CommonCard';

interface IGeneralSettingProps {}

export const GeneralSetting: React.FC<IGeneralSettingProps> = () => {
  const [form] = Form.useForm();
  const { maxLogSize, setMaxLogSize } = useGeneralSetting();
  const [messageApi, contextHolder] = message.useMessage();
  const { t } = useI18n();

  return (
    <CommonCard
      title={t('settings.general.title')}
      subTitle={t('settings.general.subTitle')}
      extra={
        <Space>
          <Button
            type="primary"
            onClick={() => {
              form.validateFields().then(() => {
                form.submit();
              });
            }}
          >
            {t('settings.general.actions.save')}
          </Button>
          <Button
            type="dashed"
            onClick={() => {
              form.resetFields();
            }}
          >
            {t('settings.general.actions.reset')}
          </Button>
        </Space>
      }
    >
      {contextHolder}
      <Form
        className="w-full"
        layout="vertical"
        form={form}
        initialValues={{
          maxLogSize: maxLogSize,
        }}
        onFinish={async ({ maxLogSize }) => {
          setMaxLogSize(maxLogSize);
          messageApi.success(t('settings.general.actions.save'));
        }}
      >
        <Typography.Title level={5} className="mb-2">
          {t('settings.general.language')}
        </Typography.Title>
        <LanguageSelector />
        <Typography.Title level={5} className="mb-2">
          {t('settings.general.maxLogSize.title')}
        </Typography.Title>
        <Typography.Paragraph className="mb-2">
          {t('settings.general.maxLogSize.description')}
        </Typography.Paragraph>

        <Form.Item
          colon={false}
          name={'maxLogSize'}
          rules={[
            {
              required: true,
              message: t('settings.general.maxLogSize.validation.required'),
            },
            {
              type: 'number',
              min: 60,
              max: 6000,
              message: t('settings.general.maxLogSize.validation.range'),
            },
          ]}
        >
          <InputNumber className="w-full" />
        </Form.Item>
      </Form>
    </CommonCard>
  );
};
