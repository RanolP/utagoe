import { styled } from 'goober';
import { Redirect, Route, Router, Switch } from 'wouter-preact';
import useLocation from 'wouter-preact/use-location';
import { DiscoverPage } from './pages/DiscoverPage';
import { PlayerPage } from './pages/PlayerPage';
import { Sidebar } from './system/Sidebar';

const Wrap = styled('div')`
  height: 100%;
  font-size: 1.5em;
  background: transparent;

  display: grid;
  grid-template-areas: 'sidebar main';
  grid-template-columns: auto 1fr;
`;

export function App(): JSX.Element {
  const [loc] = useLocation();
  return (
    <Router>
      <Switch>
        <Route path="/player">
          <PlayerPage />
        </Route>
        <Route>
          <Wrap>
            <Sidebar />
            <Switch>
              <Route path="/discover">
                <DiscoverPage />
              </Route>
              <Route path="/wiki">Wikiiii</Route>
              <Route>
                <Redirect to="/discover" />
              </Route>
            </Switch>
          </Wrap>
        </Route>
      </Switch>
    </Router>
  );
}
