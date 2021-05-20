import { styled } from 'goober';
import { Link, LinkProps } from 'wouter-preact';
import useLocation from 'wouter-preact/use-location';

const List = styled('ul')`
  margin: 0;
  padding: 0;
  list-style: none;

  height: 100%;

  grid-area: sidebar;

  display: flex;
  flex-direction: column;
  justify-content: center;

  background: #aaaaaa;

  li {
    padding: 1rem 2rem;
  }
`;

const LinkStyled = styled<{ highlight?: boolean } & LinkProps>(Link)`
  ${({ highlight }) => highlight && `font-weight: bold;`}
`;

export function Sidebar(): JSX.Element {
  const [location] = useLocation();

  return (
    <List>
      <li>
        <LinkStyled highlight={location === '/discover'} href="/discover">
          발견
        </LinkStyled>
      </li>
      <li>
        <LinkStyled highlight={location === '/wiki'} href="/wiki">
          위키
        </LinkStyled>
      </li>
    </List>
  );
}
