import {
  Applemusic,
  Bandcamp,
  Deezer,
  Niconico,
  Soundcloud,
  Spotify,
  Youtube,
} from '@icons-pack/react-simple-icons';
import Select from 'react-select';

type Platform = {
  name: string;
  id: string;
  icon: JSX.Element;
};

const Platforms: Platform[] = [
  {
    name: 'YouTube',
    id: 'youtube',
    icon: <Youtube color="#ff0000" />,
  },
  {
    name: 'Soundcloud',
    id: 'soundcloud',
    icon: <Soundcloud color="#ff3300" />,
  },
  {
    name: 'Niconico Douga',
    id: 'niconico-douga',
    icon: <Niconico color="#000000" />,
  },
  {
    name: 'Spotify',
    id: 'spotify',
    icon: <Spotify color="#1db954" />,
  },
  {
    name: 'Apple Music',
    id: 'apple-music',
    icon: <Applemusic color="#fa2d48" />,
  },
  {
    name: 'Bandcamp',
    id: 'bandcamp',
    icon: <Bandcamp color="#629aa9" />,
  },
];

function getOptionValue(props: Platform): string {
  return props.id;
}

function formatOptionLabel(props: Platform): JSX.Element {
  return (
    <div>
      {props.icon} {props.name}
    </div>
  );
}

export function DiscoverPage(): JSX.Element {
  return (
    <div>
      <h1>Discover</h1>
      <form>
        <Select
          options={Platforms}
          defaultValue={Platforms[0]}
          getOptionValue={getOptionValue}
          formatOptionLabel={formatOptionLabel}
        />
        에서
        <input />
        <button>검색</button>
      </form>
      <h3>구독 중인 아티스트의 새 곡</h3>
      <ul>
        <li>
          {Platforms[0].icon}
          <h4>하츠네미쿠의 소실 by 하츠네미쿠 · 3일 전</h4>
        </li>
        <li>
          {Platforms[1].icon}
          <h4>하츠네미쿠의 소실 by 하츠네미쿠 · 3일 전</h4>
        </li>
        <li>
          {Platforms[2].icon}
          <h4>하츠네미쿠의 소실 by 하츠네미쿠 · 3일 전</h4>
        </li>
        <li>
          {Platforms[3].icon}
          <h4>하츠네미쿠의 소실 by 하츠네미쿠 · 3일 전</h4>
        </li>
      </ul>
    </div>
  );
}
