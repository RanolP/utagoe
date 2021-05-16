import { styled } from 'goober';

const Wrap = styled('div')`
  height: 100%;
  text-align: center;
  font-size: 1.5em;
  background: #c0ffee;

  h1 {
    margin: 0;
  }
`;

export function App(): JSX.Element {
  return (
    <Wrap>
      <h1>Hello Utagoe!</h1>
    </Wrap>
  );
}
