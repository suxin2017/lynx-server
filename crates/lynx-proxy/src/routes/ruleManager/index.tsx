import { createFileRoute } from '@tanstack/react-router';
import { InterceptorPage } from './components/InterceptorPage.tsx';

export const Route = createFileRoute('/ruleManager/')({
  component: RouteComponent,
});

function RouteComponent() {
  return <InterceptorPage />;
}
