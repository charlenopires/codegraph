import { useEffect, useRef, useState } from 'react';
import * as d3 from 'd3';
import type { GraphElement } from '@/api/types';

interface GraphNode extends d3.SimulationNodeDatum {
  id: string;
  name: string;
  category: string;
  design_system: string;
  connections: number;
}

interface GraphLink extends d3.SimulationLinkDatum<GraphNode> {
  type: string;
}

interface ForceGraphProps {
  nodes: GraphElement[];
  links?: { source: string; target: string; type: string }[];
  onNodeClick?: (node: GraphElement) => void;
  width?: number;
  height?: number;
}

const DESIGN_SYSTEM_COLORS: Record<string, string> = {
  'material-ui': '#2196f3',
  'tailwind': '#38bdf8',
  'chakra': '#319795',
  'bootstrap': '#7952b3',
  'ant-design': '#1890ff',
  'shadcn': '#ffffff',
  'custom': '#a855f7',
  'unknown': '#6b7280',
};

const CATEGORY_COLORS: Record<string, string> = {
  'button': '#f59e0b',
  'input': '#10b981',
  'card': '#3b82f6',
  'modal': '#8b5cf6',
  'navigation': '#ec4899',
  'layout': '#06b6d4',
  'text': '#6b7280',
  'default': '#9ca3af',
};

export function ForceGraph({
  nodes,
  links = [],
  onNodeClick,
  width = 800,
  height = 600,
}: ForceGraphProps) {
  const svgRef = useRef<SVGSVGElement>(null);
  const [selectedNode, setSelectedNode] = useState<GraphNode | null>(null);

  useEffect(() => {
    if (!svgRef.current || nodes.length === 0) return;

    const svg = d3.select(svgRef.current);
    svg.selectAll('*').remove();

    // Create graph data
    const graphNodes: GraphNode[] = nodes.map((node) => ({
      id: node.id,
      name: node.name,
      category: node.category,
      design_system: node.design_system,
      connections: node.connections,
    }));

    const graphLinks: GraphLink[] = links.map((link) => ({
      source: link.source,
      target: link.target,
      type: link.type,
    }));

    // Create container with zoom
    const container = svg
      .append('g')
      .attr('class', 'graph-container');

    const zoom = d3.zoom<SVGSVGElement, unknown>()
      .scaleExtent([0.1, 4])
      .on('zoom', (event) => {
        container.attr('transform', event.transform);
      });

    svg.call(zoom);

    // Create force simulation
    const simulation = d3.forceSimulation(graphNodes)
      .force('link', d3.forceLink<GraphNode, GraphLink>(graphLinks)
        .id((d) => d.id)
        .distance(100))
      .force('charge', d3.forceManyBody().strength(-300))
      .force('center', d3.forceCenter(width / 2, height / 2))
      .force('collision', d3.forceCollide().radius(30));

    // Create arrow markers for directed edges
    svg.append('defs').selectAll('marker')
      .data(['arrow'])
      .join('marker')
      .attr('id', 'arrow')
      .attr('viewBox', '0 -5 10 10')
      .attr('refX', 25)
      .attr('refY', 0)
      .attr('markerWidth', 6)
      .attr('markerHeight', 6)
      .attr('orient', 'auto')
      .append('path')
      .attr('fill', 'var(--color-muted)')
      .attr('d', 'M0,-5L10,0L0,5');

    // Create links
    const link = container.append('g')
      .attr('class', 'links')
      .selectAll('line')
      .data(graphLinks)
      .join('line')
      .attr('stroke', 'var(--color-border)')
      .attr('stroke-opacity', 0.6)
      .attr('stroke-width', 1.5)
      .attr('marker-end', 'url(#arrow)');

    // Create node groups
    const node = container.append('g')
      .attr('class', 'nodes')
      .selectAll<SVGGElement, GraphNode>('g')
      .data(graphNodes)
      .join('g')
      .attr('class', 'node')
      .style('cursor', 'pointer');

    // Apply drag behavior
    const dragBehavior = d3.drag<SVGGElement, GraphNode>()
      .on('start', (event, d) => {
        if (!event.active) simulation.alphaTarget(0.3).restart();
        d.fx = d.x;
        d.fy = d.y;
      })
      .on('drag', (event, d) => {
        d.fx = event.x;
        d.fy = event.y;
      })
      .on('end', (event, d) => {
        if (!event.active) simulation.alphaTarget(0);
        d.fx = null;
        d.fy = null;
      });

    node.call(dragBehavior);

    // Add circles to nodes
    node.append('circle')
      .attr('r', (d) => Math.max(8, Math.min(20, 8 + d.connections * 2)))
      .attr('fill', (d) => DESIGN_SYSTEM_COLORS[d.design_system] || DESIGN_SYSTEM_COLORS['unknown'])
      .attr('stroke', (d) => CATEGORY_COLORS[d.category] || CATEGORY_COLORS['default'])
      .attr('stroke-width', 3)
      .attr('opacity', 0.9);

    // Add labels to nodes
    node.append('text')
      .text((d) => d.name.length > 12 ? d.name.slice(0, 12) + '...' : d.name)
      .attr('x', 0)
      .attr('y', -18)
      .attr('text-anchor', 'middle')
      .attr('fill', 'var(--color-foreground)')
      .attr('font-size', '11px')
      .attr('font-weight', '500')
      .style('pointer-events', 'none');

    // Node hover effects
    node
      .on('mouseover', function(_event, d) {
        d3.select(this).select('circle')
          .attr('stroke-width', 5)
          .attr('opacity', 1);

        // Highlight connected links
        link
          .attr('stroke-opacity', (l) =>
            (l.source as GraphNode).id === d.id || (l.target as GraphNode).id === d.id ? 1 : 0.2
          )
          .attr('stroke-width', (l) =>
            (l.source as GraphNode).id === d.id || (l.target as GraphNode).id === d.id ? 2.5 : 1.5
          );
      })
      .on('mouseout', function() {
        d3.select(this).select('circle')
          .attr('stroke-width', 3)
          .attr('opacity', 0.9);

        link
          .attr('stroke-opacity', 0.6)
          .attr('stroke-width', 1.5);
      })
      .on('click', (_event, d) => {
        setSelectedNode(d);
        if (onNodeClick) {
          onNodeClick({
            id: d.id,
            name: d.name,
            category: d.category,
            design_system: d.design_system,
            connections: d.connections,
          });
        }
      });

    // Update positions on simulation tick
    simulation.on('tick', () => {
      link
        .attr('x1', (d) => (d.source as GraphNode).x!)
        .attr('y1', (d) => (d.source as GraphNode).y!)
        .attr('x2', (d) => (d.target as GraphNode).x!)
        .attr('y2', (d) => (d.target as GraphNode).y!);

      node.attr('transform', (d) => `translate(${d.x},${d.y})`);
    });

    // Click outside to deselect
    svg.on('click', () => {
      setSelectedNode(null);
    });

    // Cleanup
    return () => {
      simulation.stop();
    };
  }, [nodes, links, width, height, onNodeClick]);

  return (
    <div className="relative w-full h-full">
      <svg
        ref={svgRef}
        width={width}
        height={height}
        className="bg-surface rounded-lg"
        style={{ width: '100%', height: '100%' }}
      />

      {/* Legend */}
      <div className="absolute top-4 left-4 bg-surface-elevated/90 backdrop-blur-sm rounded-lg p-3 text-xs space-y-2 border border-border">
        <div className="font-medium text-foreground mb-2">Design Systems</div>
        {Object.entries(DESIGN_SYSTEM_COLORS).slice(0, 5).map(([ds, color]) => (
          <div key={ds} className="flex items-center gap-2">
            <div
              className="w-3 h-3 rounded-full"
              style={{ backgroundColor: color }}
            />
            <span className="text-muted-foreground capitalize">{ds.replace('-', ' ')}</span>
          </div>
        ))}
      </div>

      {/* Selected node details */}
      {selectedNode && (
        <div className="absolute bottom-4 right-4 bg-surface-elevated/90 backdrop-blur-sm rounded-lg p-4 min-w-64 border border-border">
          <h3 className="font-medium text-foreground">{selectedNode.name}</h3>
          <div className="mt-2 space-y-1 text-sm">
            <div className="flex justify-between">
              <span className="text-muted-foreground">Category:</span>
              <span className="text-foreground capitalize">{selectedNode.category}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Design System:</span>
              <span className="text-foreground">{selectedNode.design_system || 'Unknown'}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Connections:</span>
              <span className="text-foreground">{selectedNode.connections}</span>
            </div>
          </div>
        </div>
      )}

      {/* Instructions */}
      <div className="absolute bottom-4 left-4 text-xs text-muted-foreground bg-surface-elevated/90 backdrop-blur-sm rounded-lg px-3 py-2 border border-border">
        Drag nodes to reposition • Scroll to zoom • Click node for details
      </div>
    </div>
  );
}
