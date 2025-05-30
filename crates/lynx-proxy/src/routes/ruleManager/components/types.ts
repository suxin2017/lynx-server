import {
  CaptureRule,
  HandlerRule,
  RequestRule,
} from '../../../services/generated/utoipaAxum.schemas';

export interface FormValues {
  name: string;
  description?: string;
  priority: number;
  enabled: boolean;

  // 规则类型选择
  ruleType: 'simple' | 'complex';

  // 捕获规则配置
  capture: CaptureRule;

  // 处理器配置
  handlers: HandlerRule[];

  // 向后兼容的字段（逐步淘汰）
  urlPattern?: string;
  method?: string;
  headers?: { key: string; value: string }[];
  actionType?: 'block' | 'redirect' | 'modify_request' | 'modify_response';
  redirectTarget?: string;
  requestHeaders?: { key: string; value: string }[];
  responseStatusCode?: number;
  responseBody?: string;
}

// 转换函数：将 FormValues 转换为 RequestRule
export const formValuesToRequestRule = (values: FormValues): RequestRule => {
  return {
    name: values.name,
    description: values.description || undefined,
    priority: values.priority,
    enabled: values.enabled,
    capture: values.capture,
    handlers: values.handlers || [],
  };
};

// 转换函数：将 RequestRule 转换为 FormValues
export const requestRuleToFormValues = (rule: RequestRule): FormValues => {
  return {
    name: rule.name,
    description: rule.description || undefined,
    priority: rule.priority,
    enabled: rule.enabled,
    ruleType: rule.capture.condition.type === 'simple' ? 'simple' : 'complex',
    capture: rule.capture,
    handlers: rule.handlers || [],
  };
};
