import styles from "./ImagesList.module.scss";

type Image = {
  name: string;
  author: string;
  lastModified: string;
};

const IMAGES: Image[] = [
  {
    name: "PelleSvans",
    author: "Vidde",
    lastModified: "2024-02-23T14:22:53",
  },
  {
    name: "Dallepoo",
    author: "Vidde",
    lastModified: "2023-01-21T09:19:12",
  },
  {
    name: "Leffeeeepo",
    author: "Vidde",
    lastModified: "2024-02-25T20:22:53",
  },
];

export const ImagesList = () => {
  return (
    <div className={`card`}>
      <table>
        <thead>
          <tr>
            <th colSpan={3}>
              <h3>Images</h3>
            </th>
          </tr>
          <tr>
            <th>Name</th>
            <th>Author</th>
            <th>Last modified</th>
          </tr>
        </thead>
        <tbody>
          {IMAGES.map((image) => (
            <ImageRow image={image} key={image.name} />
          ))}
        </tbody>
      </table>
    </div>
  );
};

const ImageRow = ({ image }: { image: Image }) => {
  const now = Date.now();
  const lastModified = new Date(image.lastModified);

  const diff = now - lastModified.getDate();

  return (
    <tr key={image.name}>
      <td>{image.name}</td>
      <td>{image.author}</td>
      <td>{image.lastModified}</td>
    </tr>
  );
};
