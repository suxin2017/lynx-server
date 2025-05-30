import { CardProps, Typography } from 'antd';

export const CommonCard: React.FC<
  CardProps & {
    subTitle?: string;
  }
> = ({ title, subTitle, children, extra }) => {
  return (
    <div className="h-full w-full overflow-auto rounded-xl border border-gray-300 px-4 py-4 dark:border-gray-500">
      <div className="flex items-center justify-between">
        <div>
          <Typography.Title level={3} className="m-0">
            {title}
          </Typography.Title>
          <Typography.Title
            level={5}
            className="m-0 text-gray-500 dark:text-gray-400"
          >
            {subTitle}
          </Typography.Title>
        </div>
        {extra}
      </div>
      {children}
    </div>
  );
};
