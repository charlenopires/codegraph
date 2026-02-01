import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { Layout } from '@/components/layout/Layout';
import {
  UploadPage,
  QueryPage,
  GraphPage,
  MetricsPage,
  SettingsPage,
  HelpPage,
  OntologyPage,
} from '@/pages';

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<Layout />}>
          <Route index element={<Navigate to="/upload" replace />} />
          <Route path="upload" element={<UploadPage />} />
          <Route path="query" element={<QueryPage />} />
          <Route path="graph" element={<GraphPage />} />
          <Route path="metrics" element={<MetricsPage />} />
          <Route path="ontology" element={<OntologyPage />} />
          <Route path="settings" element={<SettingsPage />} />
          <Route path="help" element={<HelpPage />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}

export default App;
