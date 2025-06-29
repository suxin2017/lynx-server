import React from 'react';
import { HandlerRuleType } from '@/services/generated/utoipaAxum.schemas';
import { BlockHandlerConfig } from './BlockHandlerConfig';
import { DelayHandlerConfig } from './DelayHandlerConfig';
import { ModifyRequestConfig } from './ModifyRequestConfig';
import { ModifyResponseConfig } from './ModifyResponseConfig';
import { LocalFileConfig } from './LocalFileConfig';
import { ProxyForwardConfig } from './ProxyForwardConfig';
import { HtmlScriptInjectorConfig } from './HtmlScriptInjectorConfig';

interface HandlerConfigProps {
  field: {
    key: number;
    name: number;
  };
  handler: HandlerRuleType;
}

export const HandlerConfig: React.FC<HandlerConfigProps> = ({
  handler,
  field,
}) => {
  // Type guard to safely access the type property
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const getHandlerType = (handlerType: any): string => {
    if (
      handlerType &&
      typeof handlerType === 'object' &&
      'type' in handlerType
    ) {
      return handlerType.type;
    }
    return 'unknown';
  };

  const handlerType = getHandlerType(handler);

  switch (handlerType) {
    case 'block':
      return <BlockHandlerConfig field={field} />;
    case 'delay':
      return <DelayHandlerConfig field={field} />;
    case 'modifyRequest':
      return <ModifyRequestConfig field={field} />;
    case 'modifyResponse':
      return <ModifyResponseConfig field={field} />;
    case 'localFile':
      return <LocalFileConfig field={field} />;
    case 'proxyForward':
      return <ProxyForwardConfig field={field} />;
    case 'htmlScriptInjector':
      return <HtmlScriptInjectorConfig field={field} />;
    default:
      return (
        <></>
      );
  }
};
