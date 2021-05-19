import { styled } from 'goober';
import { Link } from 'wouter';

const List = styled('ul')`
  margin: 0;
  padding: 0;
  list-style: none;

  height: 100%;

  grid-area: sidebar;

  display: flex;
  flex-direction: column;
  justify-content: center;

  li {
    padding: 1rem 2rem;
  }
`;

export function Sidebar(): JSX.Element {
  return (
    <List>
      <li>
        <Link href="discover">발견</Link>
      </li>
      <li>
        <Link href="wiki">위키</Link>
      </li>
    </List>
  );
}
