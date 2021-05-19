import { styled } from 'goober';
import { Redirect, Route, Router, Switch } from 'wouter-preact';
import { Sidebar } from './system/Sidebar';

const Wrap = styled('div')`
  height: 100%;
  font-size: 1.5em;
  background: transparent;

  display: grid;
  grid-template-areas: 'sidebar main';
`;

export function App(): JSX.Element {
  return (
    <Wrap>
      <Router>
        <Sidebar />
        <Switch>
          <Route path="/discover">Discoverrr</Route>
          <Route path="/wiki">Wikiiii</Route>
          <Route path="/">
            <Redirect to="/discover" />
          </Route>
          <Route>404040404</Route>
        </Switch>
      </Router>
    </Wrap>
  );
}
