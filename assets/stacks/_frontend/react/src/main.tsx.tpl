import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';

function App(): JSX.Element {
  return (
    <StrictMode>
      <h1>{{project_name}} — {{stack_id}}</h1>
    </StrictMode>
  );
}

const root = document.getElementById('root');
if (root) {
  createRoot(root).render(<App />);
}
