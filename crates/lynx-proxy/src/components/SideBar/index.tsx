import {
  RiFileListLine,
  RiMoonLine,
  RiPlanetFill,
  RiSettings2Fill,
  RiSunLine,
} from '@remixicon/react';
import { useLocation, useNavigate } from '@tanstack/react-router';
import { Button } from 'antd';
import React, { useEffect, useState } from 'react';
import { useI18n } from '@/contexts';

export const SideBar: React.FC = (_props) => {
  const navigate = useNavigate();
  const { pathname } = useLocation();
  const { t } = useI18n();
  const [theme, setTheme] = useState(() => {
    if (typeof window !== 'undefined') {
      return localStorage.getItem('theme') || 'light';
    }
    return 'light';
  });

  const topMenuConfig = [
    {
      key: '/network',
      title: t('sideBar.network'),
      icon: <RiPlanetFill className="text-slate-600" size={24} />,
    },
    {
      key: '/ruleManager',
      title: t('sideBar.rules'),
      icon: <RiFileListLine className="text-slate-600" size={24} />,
    },
  ];
  const bottomMenuConfig = [
    {
      key: '/settings',
      title: t('sideBar.settings'),
      icon: <RiSettings2Fill className="text-slate-600" size={24} />,
    },
  ];

  // 使用 @tanstack/react-router 获取当前路径
  const currentPath = pathname;

  useEffect(() => {
    if (theme === 'dark') {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
    localStorage.setItem('theme', theme);
  }, [theme]);

  const toggleTheme = () => {
    setTheme((prev) => (prev === 'light' ? 'dark' : 'light'));
  };

  return (
    <div className="flex w-14 flex-col justify-between shadow-xs shadow-slate-400">
      <div>
        {topMenuConfig.map((item) => (
          <Button
            key={item.key}
            type="text"
            className={`flex h-14 w-full items-center justify-items-center ${
              currentPath === item.key ? 'bg-zinc-200 dark:bg-zinc-800' : ''
            }`}
            onClick={() => {
              navigate({
                to: item.key,
              });
            }}
            icon={item.icon}
            title={item.title}
          />
        ))}
      </div>
      <div className="flex flex-col">
        <Button
          type="text"
          className="flex h-14 w-full items-center justify-items-center"
          onClick={toggleTheme}
          icon={
            theme === 'dark' ? (
              <RiMoonLine className="text-slate-600" size={24} />
            ) : (
              <RiSunLine className="text-slate-600" size={24} />
            )
          }
        />
        {bottomMenuConfig.map((item) => (
          <Button
            key={item.key}
            type="text"
            className={`flex h-14 w-full items-center justify-items-center ${
              currentPath === item.key ? 'bg-zinc-200 dark:bg-zinc-800' : ''
            }`}
            onClick={() => {
              navigate({
                to: item.key,
              });
            }}
            icon={item.icon}
            title={item.title}
          />
        ))}
      </div>
    </div>
  );
};
