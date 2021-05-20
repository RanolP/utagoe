import { css, styled } from 'goober';
import {
  Lock,
  Play,
  Search,
  Share2,
  SkipBack,
  SkipForward,
  Unlock,
} from 'preact-feather';
import { useCallback, useState } from 'preact/hooks';
import { Button } from '../system/ButtonBase';

const Wrap = styled('div')`
  background: #ffffffaa;
  width: 100%;
  height: 100%;

  display: grid;

  grid-template-columns: auto 1fr auto;
  grid-template-rows: auto 1fr auto auto;
  grid-template-areas:
    'lock spacing share'
    'lyric lyric lyric'
    'control control control'
    'time time time'
    'search search search';
`;

const LockButton = styled(Button)`
  grid-area: lock;
`;

const ShareButton = styled(Button)`
  grid-area: share;
`;

const LyricWrap = styled('div')`
  grid-area: lyric;
`;

const Lyric = styled<{ highlight?: boolean }>('p')`
  width: fit-content;
  margin: 0 auto;

  ${({ highlight }) => highlight && `font-weight: bold;`}
`;

const Control = styled('div')`
  grid-area: control;

  display: flex;
  flex-direction: row;
  justify-content: center;
`;

const Time = styled('div')`
  grid-area: time;

  user-select: none;
  cursor: default;
`;

const SearchForm = styled('form')`
  grid-area: search;

  display: flex;
  flex-direction: row;

  input {
    flex: 1;
    margin: 0;
    padding: 0.2rem 0.4rem;

    background: transparent;
    border: none;
    border-top: 1px solid black;
  }

  button {
    border: none;
    background: #c0ffee;
    margin: 0;
  }
`;

export function PlayerPage(): JSX.Element {
  const [locked, setLocked] = useState(false);

  const toggleLock = useCallback(() => {
    setLocked((prev) => !prev);
  }, []);

  const dragProps = !locked ? { 'data-tauri-drag-region': '' } : {};

  return (
    <Wrap {...dragProps}>
      <LockButton onClick={toggleLock}>
        {locked ? <Lock /> : <Unlock />}
      </LockButton>
      <ShareButton>
        <Share2 />
      </ShareButton>
      <LyricWrap {...dragProps}>
        <Lyric highlight={true}>가사 지금 줄</Lyric>
        <Lyric>가사 다음 줄</Lyric>
      </LyricWrap>
      <Control>
        <button>
          <SkipBack />
        </button>
        <button>
          <Play />
        </button>
        <button>
          <SkipForward />
        </button>
      </Control>
      <Time>
        <span {...dragProps}>남은 시간</span>
        <span {...dragProps}>/</span>
        <span {...dragProps}>전체 시간</span>
      </Time>
      <SearchForm>
        <input />
        <button>
          <Search />
        </button>
      </SearchForm>
    </Wrap>
  );
}
