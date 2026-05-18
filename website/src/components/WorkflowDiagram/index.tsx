import type {ReactNode} from 'react';
import Mermaid from '@theme/Mermaid';
import Translate from '@docusaurus/Translate';
import styles from './styles.module.css';

const DIAGRAM = `flowchart LR
  classDef human fill:#e0f2fe,stroke:#0284c7,color:#0c4a6e
  classDef agent fill:#fef3c7,stroke:#d97706,color:#78350f
  classDef framework fill:#dcfce7,stroke:#16a34a,color:#14532d

  subgraph H["Human"]
    direction LR
    spec["Spec / intent"]:::human
    review["Review & merge"]:::human
  end

  subgraph A["Agent"]
    direction LR
    plan["Plan & implement"]:::agent
    audit["Self-audit"]:::agent
  end

  subgraph F["StrayMark"]
    direction LR
    charter["Charter"]:::framework
    ailog["AILOG"]:::framework
    tde["TDE"]:::framework
    trail["Audit trail"]:::framework
  end

  spec --> charter --> plan --> ailog --> audit --> tde --> review --> trail
`;

export default function WorkflowDiagram(): ReactNode {
  return (
    <section className={styles.section}>
      <div className={styles.inner}>
        <h2 className={styles.title}>
          <Translate id="workflow.title" description="Workflow section title">
            The workflow
          </Translate>
        </h2>
        <p className={styles.caption}>
          <Translate id="workflow.caption" description="Workflow section caption">
            Every step leaves a versioned artifact in the repo. No external systems, no implicit decisions.
          </Translate>
        </p>
        <div className={styles.diagram}>
          <Mermaid value={DIAGRAM} />
        </div>
      </div>
    </section>
  );
}
